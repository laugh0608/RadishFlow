//! Isolated Studio session for qualifying and saving one explicit association edit.

use super::{
    H3AssociationPreview, H3AssociationSaveTarget, H3FlowReview, H3FlowStream,
    PreparedH3FlowAssociationSave, Side, bundled_water, inputs_changed, parse_project, rejected,
};
use crate::{
    WorkspaceExecutionOutcome, WorkspaceRunOwner, WorkspaceRunPhase, WorkspaceRunSubmission,
};
use rf_model::equipment::{ConcentricTubeFlowAssociation, ConcentricTubeRatingTask};
use rf_store::project_v4::StoredFlowPlan;
use rf_types::{RfError, RfResult};
use rf_ui::{
    FlowRunDisposition, FlowRunId, engineering::EquipmentCaseKey, variable_browser::ObjectId,
};
use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Debug, Default)]
pub enum H3AssociationPreviewState {
    #[default]
    Missing,
    Cancelled,
    Rejected(RfError),
    Ready(Box<H3AssociationPreview>),
}

#[derive(Debug)]
struct PendingPreview {
    id: FlowRunId,
    candidate: Option<(EquipmentCaseKey, ConcentricTubeFlowAssociation)>,
}

#[derive(Debug)]
pub struct StudioH3AssociationEditor {
    review: H3FlowReview,
    pending: Option<PendingPreview>,
    preview: H3AssociationPreviewState,
    prepared: Option<PreparedH3FlowAssociationSave>,
}

impl StudioH3AssociationEditor {
    pub fn supports_original(contents: &str) -> bool {
        rf_store::project_v3::probe_version(contents) == Ok(rf_store::project_v4::VERSION)
            && parse_project(contents).is_ok_and(|project| {
                matches!(project.steady_state_plan, StoredFlowPlan::Acyclic)
                    && project.object_identities.is_some()
                    && bundled_water::supports_package(
                        project.document.flowsheet.property_package_id(),
                    )
            })
    }

    pub fn open(path: &Path, original: &str) -> RfResult<Self> {
        let editor = Self::load(path)?;
        if editor.review.saved.original_json != original {
            return Err(rejected(
                "source_changed",
                "the source file changed after opening",
            ));
        }
        Ok(editor)
    }

    fn load(path: &Path) -> RfResult<Self> {
        let review = H3FlowReview::open(path, bundled_water::reviewed_qualification())?;
        if !bundled_water::supports_package(
            review.workspace().document.flowsheet.property_package_id(),
        ) {
            return Err(rejected(
                "package_unqualified",
                "the saved flowsheet requires the approved pure-water package",
            ));
        }
        Ok(Self {
            review,
            pending: None,
            preview: H3AssociationPreviewState::Missing,
            prepared: None,
        })
    }

    pub fn reload(&mut self) -> RfResult<()> {
        if self.pending.is_some() {
            return Err(rejected(
                "preview_busy",
                "wait for the preview run to finish",
            ));
        }
        let next = Self::load(self.review.source_path()?)?;
        if next.review.saved.metadata().document_id != self.review.saved.metadata().document_id {
            return Err(rejected(
                "source_changed",
                "the source now contains a different project",
            ));
        }
        *self = next;
        Ok(())
    }

    pub fn cases(&self) -> impl Iterator<Item = &ConcentricTubeRatingTask> {
        self.review.cases()
    }

    pub fn case(&self, member: &EquipmentCaseKey) -> Option<&ConcentricTubeRatingTask> {
        self.review.saved.case(member)
    }

    pub fn display_units(&self) -> &rf_types::units::DisplayUnitSet {
        self.review.saved.display_units()
    }

    pub fn streams(&self) -> impl Iterator<Item = (H3FlowStream, &str)> {
        let workspace = self.review.workspace();
        workspace
            .document
            .flowsheet
            .streams
            .iter()
            .filter_map(|(id, stream)| {
                let creation_id = workspace.object_creation_id(&ObjectId::Stream(id.clone()))?;
                Some((
                    H3FlowStream {
                        id: id.clone(),
                        creation_id: creation_id.clone(),
                    },
                    stream.name.as_str(),
                ))
            })
    }

    pub fn preview(&self) -> &H3AssociationPreviewState {
        &self.preview
    }

    pub fn activity(&self, owner: &WorkspaceRunOwner) -> Option<WorkspaceRunPhase> {
        let pending = self.pending.as_ref()?;
        owner
            .state()
            .filter(|state| state.run_id == pending.id)
            .map(|state| state.phase)
    }

    pub fn clear_preview(&mut self) {
        if let Some(pending) = &mut self.pending {
            pending.candidate = None;
        }
        self.prepared = None;
        self.preview = H3AssociationPreviewState::Missing;
    }

    pub fn start(
        &mut self,
        owner: &mut WorkspaceRunOwner,
        member: EquipmentCaseKey,
        pair: ConcentricTubeFlowAssociation,
        now: SystemTime,
    ) -> RfResult<()> {
        if self.pending.is_some() || !owner.is_idle() {
            return Err(rejected("preview_busy", "wait for the current H3 flow run"));
        }
        self.clear_preview();
        self.review.check_source_bytes()?;
        let task = self.review.saved.case(&member).ok_or_else(inputs_changed)?;
        if task.case.flow_association.as_ref() == Some(&pair) {
            return Err(rejected(
                "association_unchanged",
                "the saved association is unchanged",
            ));
        }
        if pair.hot.id == pair.cold.id {
            return Err(
                rejected("same_stream", "hot and cold require distinct streams")
                    .with_related_stream_id(pair.hot.id),
            );
        }
        self.review.check_stream(&pair.hot, Side::Hot)?;
        self.review.check_stream(&pair.cold, Side::Cold)?;
        bundled_water::check_association(
            &self.review.qualification,
            self.review
                .workspace()
                .document
                .flowsheet
                .property_package_id(),
            self.review.saved.basis(),
            &task.case,
            Some(&pair),
        )?;
        let request = self.review.solve_request()?;
        let provider = bundled_water::provider(&self.review.qualification)?;
        let WorkspaceRunSubmission::Accepted(id) =
            owner.submit_local(&mut self.review.flow, request, provider, now)?
        else {
            return Err(rejected(
                "flow_unavailable",
                "the H3 preview run was not accepted",
            ));
        };
        self.pending = Some(PendingPreview {
            id,
            candidate: Some((member, pair)),
        });
        Ok(())
    }

    pub fn cancel(&mut self, owner: &mut WorkspaceRunOwner) {
        if let Some(pending) = &self.pending {
            owner.cancel(&mut self.review.flow, &pending.id);
        }
    }

    pub fn detach(&mut self, owner: &mut WorkspaceRunOwner) {
        if self.pending.take().is_some() {
            owner.detach(&mut self.review.flow);
        }
        self.clear_preview();
    }

    pub fn poll(&mut self, owner: &mut WorkspaceRunOwner, now: SystemTime) -> bool {
        let Some(pending) = &self.pending else {
            return owner.poll_detached(now).is_some();
        };
        let Some(summary) = owner.poll(&mut self.review.flow, now) else {
            return false;
        };
        if summary.run_id != pending.id {
            return false;
        }
        let pending = self.pending.take().expect("matching preview run");
        let Some((member, pair)) = pending.candidate else {
            return true;
        };
        self.preview = if summary.cancellation.is_some()
            || summary.disposition != FlowRunDisposition::Applied
            || summary.outcome == WorkspaceExecutionOutcome::Cancelled
        {
            H3AssociationPreviewState::Cancelled
        } else if summary.outcome == WorkspaceExecutionOutcome::Succeeded {
            match self
                .review
                .check_source_bytes()
                .and_then(|()| self.review.preview_association_edit(member, pair))
            {
                Ok(preview) => H3AssociationPreviewState::Ready(Box::new(preview)),
                Err(error) => H3AssociationPreviewState::Rejected(error),
            }
        } else {
            H3AssociationPreviewState::Rejected(self.review.flow_failure(&summary))
        };
        true
    }

    pub fn prepare_save(&mut self, target: H3AssociationSaveTarget<'_>) -> RfResult<()> {
        self.prepared = None;
        let H3AssociationPreviewState::Ready(preview) = &self.preview else {
            return Err(rejected(
                "preview_required",
                "run and preview the proposed association first",
            ));
        };
        self.prepared = Some(self.review.prepare_association_edit_save(preview, target)?);
        Ok(())
    }

    pub fn prepared(&self) -> Option<&PreparedH3FlowAssociationSave> {
        self.prepared.as_ref()
    }

    pub fn cancel_save(&mut self) {
        self.prepared = None;
    }

    pub fn save(&mut self) -> RfResult<PathBuf> {
        let prepared = self.prepared.take().ok_or_else(inputs_changed)?;
        self.review
            .save_association(&prepared, prepared.requires_upgrade())?;
        self.clear_preview();
        Ok(prepared.path().to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WorkspaceRunCancelReason;
    use rf_ui::FlowRunDiscardReason;

    fn drain(
        editor: &mut StudioH3AssociationEditor,
        owner: &mut WorkspaceRunOwner,
        now: SystemTime,
    ) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !owner.is_idle() {
            editor.poll(owner, now);
            assert!(
                std::time::Instant::now() < deadline,
                "preview worker did not exit"
            );
            std::thread::yield_now();
        }
    }

    #[test]
    fn h3_association_preview_rejects_completed_payloads_after_cancel_and_reopen() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/flowsheets/h3-linked-water-40.rfproj.json");
        let bytes = std::fs::read_to_string(&source).unwrap();
        let mut editor = StudioH3AssociationEditor::open(&source, &bytes).unwrap();
        let task = editor.cases().next().unwrap();
        let member = EquipmentCaseKey {
            task_id: task.id.clone(),
            case_id: task.case.id.clone(),
        };
        let saved = task.case.flow_association.as_ref().unwrap();
        let changed = ConcentricTubeFlowAssociation {
            hot: saved.cold.clone(),
            cold: saved.hot.clone(),
        };
        let mut owner = WorkspaceRunOwner::new();
        let now = SystemTime::now();

        editor
            .start(&mut owner, member.clone(), changed.clone(), now)
            .unwrap();
        owner.wait_for_unaccepted_worker_result(now);
        editor.cancel(&mut owner);
        drain(&mut editor, &mut owner, now);
        assert!(matches!(
            editor.preview(),
            H3AssociationPreviewState::Cancelled
        ));
        assert!(
            editor
                .prepare_save(H3AssociationSaveTarget::Source)
                .is_err()
        );
        let cancelled = owner.history().back().unwrap();
        assert_eq!(cancelled.outcome, WorkspaceExecutionOutcome::Succeeded);
        assert_eq!(
            cancelled.cancellation,
            Some(WorkspaceRunCancelReason::UserRequested)
        );
        assert!(matches!(
            cancelled.disposition,
            FlowRunDisposition::Discarded(_)
        ));

        editor
            .start(&mut owner, member.clone(), changed.clone(), now)
            .unwrap();
        owner.wait_for_unaccepted_worker_result(now);
        editor.detach(&mut owner);
        let mut reopened = StudioH3AssociationEditor::open(&source, &bytes).unwrap();
        assert!(
            reopened
                .start(&mut owner, member.clone(), changed.clone(), now)
                .is_err()
        );
        drain(&mut reopened, &mut owner, now);
        assert!(matches!(
            reopened.preview(),
            H3AssociationPreviewState::Missing
        ));
        assert_eq!(
            owner.history().back().unwrap().disposition,
            FlowRunDisposition::Discarded(FlowRunDiscardReason::DifferentSession)
        );

        reopened.start(&mut owner, member, changed, now).unwrap();
        owner.wait_for_unaccepted_worker_result(now);
        drain(&mut reopened, &mut owner, now);
        assert!(matches!(
            reopened.preview(),
            H3AssociationPreviewState::Ready(_)
        ));
        assert_eq!(std::fs::read_to_string(&source).unwrap(), bytes);
    }
}
