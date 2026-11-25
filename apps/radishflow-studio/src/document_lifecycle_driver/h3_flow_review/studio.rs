//! Session state for the Studio linked H3 review. Its worker owner outlives the view.

#[cfg(test)]
mod tests;

use super::{H3FlowPreview, H3FlowRating, H3FlowReview, H3PureWaterQualification, bundled_water};
use crate::{
    WorkspaceExecutionOutcome, WorkspaceRunOwner, WorkspaceRunPhase, WorkspaceRunSubmission,
    WorkspaceRunSummary,
};
use rf_model::{engineering::EngineeringBasis, equipment::ConcentricTubeRatingTask};
use rf_store::{StoredDocumentMetadata, project_v4::StoredProjectV4};
use rf_types::{RfError, RfResult, StreamId, units::DisplayUnitSet};
use rf_ui::{
    FlowRunDisposition, FlowRunId,
    engineering::{EquipmentCaseKey, concentric_tube::ConcentricTubeRating},
    result_query::FlowResultView,
};
use std::{collections::BTreeMap, path::Path, time::SystemTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum H3FlowReviewPhase {
    Flow,
    Rating,
    Stopping,
}

#[derive(Debug)]
pub enum H3FlowReviewResult<'a> {
    Missing,
    Stale,
    Cancelled,
    Blocked(&'a RfError),
    Failed(&'a RfError),
    Current {
        rating: &'a ConcentricTubeRating,
        preview: &'a H3FlowPreview,
    },
}

#[derive(Debug)]
enum ReviewRecord {
    Stale,
    Cancelled,
    Blocked(RfError),
    Failed(RfError),
}

#[derive(Debug)]
enum ReviewWork {
    Flow(FlowRunId),
    Rating,
}

#[derive(Debug)]
struct Pending {
    member: EquipmentCaseKey,
    work: ReviewWork,
}

#[derive(Debug)]
pub struct StudioH3FlowReview {
    review: H3FlowReview,
    current: Option<H3FlowRating>,
    records: BTreeMap<EquipmentCaseKey, ReviewRecord>,
    pending: Option<Pending>,
}

impl StudioH3FlowReview {
    pub(in crate::document_lifecycle_driver) fn open(
        path: &Path,
        project: StoredProjectV4,
        contents: &str,
    ) -> RfResult<Self> {
        let review = H3FlowReview::from_saved_project(
            path,
            project,
            contents,
            bundled_water::reviewed_qualification(),
        )?;
        Ok(Self {
            review,
            current: None,
            records: BTreeMap::new(),
            pending: None,
        })
    }

    pub fn original_json(&self) -> &str {
        &self.review.saved.original_json
    }

    pub fn metadata(&self) -> &StoredDocumentMetadata {
        self.review.saved.metadata()
    }

    pub fn revision(&self) -> u64 {
        self.review.saved.revision
    }

    pub fn display_units(&self) -> &DisplayUnitSet {
        self.review.saved.display_units()
    }

    pub fn basis(&self) -> &EngineeringBasis {
        self.review.saved.basis()
    }

    pub fn qualification(&self) -> &H3PureWaterQualification {
        &self.review.qualification
    }

    pub fn qualification_identity(&self) -> (&'static str, u64) {
        (
            bundled_water::QUALIFICATION_ID,
            bundled_water::QUALIFICATION_REVISION,
        )
    }

    pub fn stream_name(&self, id: &StreamId) -> Option<&str> {
        self.review
            .workspace()
            .document
            .flowsheet
            .streams
            .get(id)
            .map(|stream| stream.name.as_str())
    }

    pub fn cases(&self) -> impl Iterator<Item = &ConcentricTubeRatingTask> {
        self.review
            .cases()
            .filter(|task| task.case.flow_association.is_some())
    }

    pub fn case(&self, member: &EquipmentCaseKey) -> Option<&ConcentricTubeRatingTask> {
        self.review
            .saved
            .case(member)
            .filter(|task| task.case.flow_association.is_some())
    }

    pub fn result(&self, member: &EquipmentCaseKey) -> H3FlowReviewResult<'_> {
        if let Some(current) = &self.current
            && current.preview.member() == member
        {
            return match self.review.current_rating(current) {
                Ok(rating) => H3FlowReviewResult::Current {
                    rating,
                    preview: &current.preview,
                },
                Err(_) => H3FlowReviewResult::Stale,
            };
        }
        match self.records.get(member) {
            None => H3FlowReviewResult::Missing,
            Some(ReviewRecord::Stale) => H3FlowReviewResult::Stale,
            Some(ReviewRecord::Cancelled) => H3FlowReviewResult::Cancelled,
            Some(ReviewRecord::Blocked(error)) => H3FlowReviewResult::Blocked(error),
            Some(ReviewRecord::Failed(error)) => H3FlowReviewResult::Failed(error),
        }
    }

    pub fn activity(
        &self,
        owner: &WorkspaceRunOwner,
    ) -> Option<(&EquipmentCaseKey, H3FlowReviewPhase)> {
        let pending = self.pending.as_ref()?;
        let phase = match &pending.work {
            ReviewWork::Flow(id)
                if owner.state().is_some_and(|run| {
                    run.run_id == *id && run.phase == WorkspaceRunPhase::CancelRequested
                }) =>
            {
                H3FlowReviewPhase::Stopping
            }
            ReviewWork::Flow(_) => H3FlowReviewPhase::Flow,
            ReviewWork::Rating => H3FlowReviewPhase::Rating,
        };
        Some((&pending.member, phase))
    }

    pub fn start(
        &mut self,
        owner: &mut WorkspaceRunOwner,
        member: &EquipmentCaseKey,
        now: SystemTime,
    ) -> bool {
        if self.pending.is_some() || !owner.is_idle() {
            return false;
        }
        let Some(task) = self.case(member) else {
            return false;
        };
        let qualified = bundled_water::check_case(
            &self.review.qualification,
            self.review
                .workspace()
                .document
                .flowsheet
                .property_package_id(),
            self.review.saved.basis(),
            &task.case,
        );
        let provider =
            match qualified.and_then(|()| bundled_water::provider(&self.review.qualification)) {
                Ok(provider) => provider,
                Err(error) => {
                    self.records
                        .insert(member.clone(), ReviewRecord::Blocked(error));
                    return false;
                }
            };
        let submission = self
            .review
            .solve_request()
            .and_then(|request| owner.submit_local(&mut self.review.flow, request, provider, now));
        let id = match submission {
            Ok(WorkspaceRunSubmission::Accepted(id)) => id,
            Ok(_) => return false,
            Err(error) => {
                self.records
                    .insert(member.clone(), ReviewRecord::Failed(error));
                return false;
            }
        };
        if let Some(previous) = self.current.take() {
            self.records
                .insert(previous.preview.member().clone(), ReviewRecord::Stale);
        }
        self.records.remove(member);
        self.pending = Some(Pending {
            member: member.clone(),
            work: ReviewWork::Flow(id),
        });
        true
    }

    pub fn cancel(&mut self, owner: &mut WorkspaceRunOwner) {
        let Some(pending) = &self.pending else { return };
        match &pending.work {
            ReviewWork::Flow(id) => {
                owner.cancel(&mut self.review.flow, id);
            }
            ReviewWork::Rating => {
                let pending = self.pending.take().expect("rating task");
                self.records.insert(pending.member, ReviewRecord::Cancelled);
            }
        }
    }

    pub fn detach(&mut self, owner: &mut WorkspaceRunOwner) {
        if let Some(Pending {
            work: ReviewWork::Flow(_),
            ..
        }) = self.pending.take()
        {
            owner.detach(&mut self.review.flow);
        }
    }

    /// Poll after processing view actions so cancelling or closing revokes an unreceived completion.
    pub fn poll(&mut self, owner: &mut WorkspaceRunOwner, now: SystemTime) -> bool {
        let Some(pending) = &self.pending else {
            return owner.poll_detached(now).is_some();
        };
        match &pending.work {
            ReviewWork::Flow(id) => {
                let Some(summary) = owner.poll(&mut self.review.flow, now) else {
                    return false;
                };
                if summary.run_id != *id {
                    return false;
                }
                if summary.cancellation.is_none()
                    && summary.disposition == FlowRunDisposition::Applied
                    && summary.outcome == WorkspaceExecutionOutcome::Succeeded
                {
                    self.pending.as_mut().expect("flow task").work = ReviewWork::Rating;
                } else {
                    let pending = self.pending.take().expect("flow task");
                    let cancelled = summary.cancellation.is_some()
                        || summary.disposition != FlowRunDisposition::Applied
                        || summary.outcome == WorkspaceExecutionOutcome::Cancelled;
                    let record = if cancelled {
                        ReviewRecord::Cancelled
                    } else {
                        ReviewRecord::Failed(self.review.flow_failure(&summary))
                    };
                    self.records.insert(pending.member, record);
                }
            }
            ReviewWork::Rating => self.finish_rating(),
        }
        true
    }

    fn finish_rating(&mut self) {
        let pending = self.pending.take().expect("rating task");
        let preview = match self.review.preview_saved(pending.member.clone()) {
            Ok(preview) => preview,
            Err(error) => {
                self.records
                    .insert(pending.member, ReviewRecord::Blocked(error));
                return;
            }
        };
        match self.review.rate(preview) {
            Ok(rating) => self.current = Some(rating),
            Err(error) => {
                self.records
                    .insert(pending.member, ReviewRecord::Failed(error));
            }
        }
    }
}

impl H3FlowReview {
    pub(super) fn flow_failure(&self, summary: &WorkspaceRunSummary) -> RfError {
        if let FlowResultView::Failed { diagnostic, .. } = self.workspace().flow_result() {
            let mut error = RfError::invalid_input(&diagnostic.primary_message);
            if let Some(code) = &diagnostic.primary_code {
                error = error.with_diagnostic_code(code);
            }
            for stream in &diagnostic.related_stream_ids {
                error = error.with_related_stream_id(stream.clone());
            }
            return error;
        }
        RfError::invalid_input("the flowsheet run did not yield an accepted result")
            .with_diagnostic_code(
                summary
                    .diagnostic_code
                    .as_deref()
                    .unwrap_or("equipment.h3.flow.flow_unavailable"),
            )
    }
}
