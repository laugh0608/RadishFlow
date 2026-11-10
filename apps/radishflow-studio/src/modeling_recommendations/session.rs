use super::*;
use crate::modeling_specifications::ModelingCheckSession;
use rf_ui::composition_commands::CompositionNormalizationRequest;
use rf_ui::{AppState, DateTimeUtc, FlowsheetDocument};

/// One owner per open workspace; the shell shares this A1 check owner across
/// problems and recommendations, without a second independent problem projection.
#[derive(Debug)]
pub struct ModelingRecommendationSession {
    plan_generation: Option<rf_ui::flow_plan::FlowPlanGeneration>,
    checks: ModelingCheckSession,
    identity: SpecificationReportIdentity,
    owner: Arc<()>,
    sequence: u64,
    entries: Vec<RecommendationEntry>,
    ignored: Vec<RecommendationKey>,
    previews: Vec<(RecommendationKey, u64)>,
    last_receipt: Option<RecommendationReceipt>,
}

impl ModelingRecommendationSession {
    pub fn new(document: &FlowsheetDocument) -> Self {
        Self::from_checks(ModelingCheckSession::new(document))
    }

    pub fn from_checks(checks: ModelingCheckSession) -> Self {
        Self {
            plan_generation: None,
            identity: checks.identity().clone(),
            checks,
            owner: Arc::new(()),
            sequence: 0,
            entries: Vec::new(),
            ignored: Vec::new(),
            previews: Vec::new(),
            last_receipt: None,
        }
    }

    pub fn checks(&self) -> &ModelingCheckSession {
        &self.checks
    }
    pub fn entries(&self) -> &[RecommendationEntry] {
        &self.entries
    }
    pub fn last_receipt(&self) -> Option<&RecommendationReceipt> {
        self.last_receipt.as_ref()
    }

    /// Required even when reopening the same persisted project at the same revision.
    pub fn replace_document(&mut self, document: &FlowsheetDocument) {
        self.checks.replace_document(document);
        self.owner = Arc::new(());
        self.last_receipt = None;
        self.synchronize_identity();
    }

    fn synchronize_identity(&mut self) {
        let generation = self.checks.plan_generation();
        if self.plan_generation.as_ref() != generation {
            self.plan_generation = generation.cloned();
            self.owner = Arc::new(());
            self.entries.clear();
            self.previews.clear();
            self.ignored.clear();
        }
        let identity = self.checks.identity();
        if &self.identity != identity {
            if self.identity.document_id != identity.document_id
                || self.identity.session != identity.session
            {
                self.owner = Arc::new(());
                self.last_receipt = None;
            }
            self.identity = identity.clone();
            self.entries.clear();
            self.ignored.clear();
            self.previews.clear();
        }
    }

    /// Refreshes ephemeral state only; the application is immutable throughout.
    pub fn refresh(&mut self, app: &AppState) {
        self.checks.refresh_workspace(&app.workspace);
        self.synchronize_identity();
        self.entries = self
            .checks
            .current_report()
            .map(|report| {
                candidates::recommendations(app, report)
                    .into_iter()
                    .map(|recommendation| {
                        let visibility = if self.ignored.contains(&recommendation.key) {
                            RecommendationVisibility::Ignored
                        } else {
                            RecommendationVisibility::Available
                        };
                        RecommendationEntry {
                            recommendation,
                            visibility,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
    }

    pub fn recheck(&mut self, app: &AppState) {
        self.checks.recheck_workspace(&app.workspace);
        self.refresh(app);
    }

    fn current(
        &self,
        key: &RecommendationKey,
    ) -> Result<&RecommendationEntry, RecommendationError> {
        if self.checks.current_report().is_none() {
            return Err(RecommendationError::ChecksUnavailable(
                self.checks.phase().clone(),
            ));
        }
        let mut matches = self
            .entries
            .iter()
            .filter(|entry| &entry.recommendation.key == key);
        let entry = matches.next().ok_or(RecommendationError::Unavailable)?;
        if matches.next().is_some() {
            return Err(RecommendationError::Ambiguous);
        }
        Ok(entry)
    }

    pub fn set_ignored(
        &mut self,
        app: &AppState,
        key: &RecommendationKey,
        ignored: bool,
    ) -> Result<(), RecommendationError> {
        self.refresh(app);
        self.current(key)?;
        self.ignored.retain(|old| old != key);
        if ignored {
            self.ignored.push(key.clone());
        }
        // Previously reviewed content must be explicitly reviewed again after hiding.
        self.previews.retain(|(prepared, _)| prepared != key);
        self.refresh(app);
        Ok(())
    }

    /// Returns a stable destination; the GUI owns navigation and keeps all drafts.
    pub fn navigation_target(
        &mut self,
        app: &AppState,
        key: &RecommendationKey,
    ) -> Result<SpecificationTarget, RecommendationError> {
        self.refresh(app);
        match &self.current(key)?.recommendation.action {
            RecommendedAction::Locate(target) => Ok(target.clone()),
            _ => Err(RecommendationError::Unavailable),
        }
    }

    /// The UI may show this preview while drafts exist, but accept remains blocked.
    pub fn prepare(
        &mut self,
        app: &AppState,
        key: &RecommendationKey,
    ) -> Result<RecommendationPreview, RecommendationError> {
        self.refresh(app);
        let entry = self.current(key)?;
        if entry.visibility == RecommendationVisibility::Ignored {
            return Err(RecommendationError::Ignored);
        }
        if matches!(entry.recommendation.action, RecommendedAction::Locate(_)) {
            return Err(RecommendationError::NavigationOnly);
        }
        let recommendation = entry.recommendation.clone();
        self.sequence = self
            .sequence
            .checked_add(1)
            .expect("recommendation preview sequence exhausted");
        self.previews.retain(|(old, _)| old != key);
        self.previews.push((key.clone(), self.sequence));
        Ok(RecommendationPreview {
            owner: self.owner.clone(),
            sequence: self.sequence,
            recommendation,
        })
    }

    pub fn accept(
        &mut self,
        app: &mut AppState,
        preview: &RecommendationPreview,
        changed_at: DateTimeUtc,
    ) -> Result<RecommendationReceipt, RecommendationError> {
        self.accept_with_recheck(app, preview, changed_at, |checks, workspace| {
            checks.recheck_workspace(workspace)
        })
    }

    /// The UI uses the same checks to explain why a preview cannot be accepted.
    /// Only session projections are refreshed; application inputs remain immutable.
    pub fn validate_preview(
        &mut self,
        app: &AppState,
        preview: &RecommendationPreview,
    ) -> Result<(), RecommendationError> {
        use RecommendationError::*;
        self.refresh(app);
        if preview.recommendation.identity.document_id != self.identity.document_id {
            return Err(DifferentDocument);
        }
        if !Arc::ptr_eq(&self.owner, &preview.owner) {
            return Err(DifferentSession);
        }
        let expected = preview.recommendation.identity.input_revision;
        let actual = self.identity.input_revision;
        if expected != actual {
            return Err(StaleRevision { expected, actual });
        }
        if !self.previews.iter().any(|(key, sequence)| {
            key == &preview.recommendation.key && *sequence == preview.sequence
        }) {
            return Err(PreviewExpired);
        }
        let entry = self.current(&preview.recommendation.key)?;
        if entry.visibility == RecommendationVisibility::Ignored {
            return Err(Ignored);
        }
        if entry.recommendation != preview.recommendation {
            return Err(PreviewChanged);
        }
        if app.workspace.drafts.pending_count() > 0 {
            return Err(PendingDrafts);
        }
        if app.workspace.canvas_interaction.pending_edit.is_some() {
            return Err(PendingCanvasEdit);
        }
        Ok(())
    }

    pub(super) fn accept_with_recheck(
        &mut self,
        app: &mut AppState,
        preview: &RecommendationPreview,
        changed_at: DateTimeUtc,
        recheck: impl FnOnce(&mut ModelingCheckSession, &rf_ui::WorkspaceState),
    ) -> Result<RecommendationReceipt, RecommendationError> {
        self.validate_preview(app, preview)?;
        let entry = self.current(&preview.recommendation.key)?;
        let actual = self.identity.input_revision;
        use RecommendationError::*;
        let command = match &entry.recommendation.action {
            RecommendedAction::Locate(_) => return Err(NavigationOnly),
            RecommendedAction::NormalizeComposition(normalization) => {
                app.normalize_committed_stream_composition(
                    CompositionNormalizationRequest {
                        document_id: self.identity.document_id.clone(),
                        expected_revision: actual,
                        preview: normalization.clone(),
                    },
                    changed_at,
                )
                .map_err(Normalization)?
                .command
            }
            RecommendedAction::MaterialConnection(connection) => {
                use crate::flow_actions::{
                    FlowAction, FlowActionRequest, FlowEditAccess, FlowEditContext,
                    execute_flow_action,
                };
                let request = FlowActionRequest {
                    expected: FlowEditContext::capture(&app.workspace),
                    action: FlowAction::Connect(crate::modeling_actions::connection_target(
                        connection,
                    )),
                };
                execute_flow_action(app, FlowEditAccess::Editable, &request, changed_at)
                    .map_err(|error| Rejected(rf_types::RfError::invalid_input(error.to_string())))?
                    .command
                    .expect("connection commits a command")
            }
        };
        // From here the edit has committed. Recheck failure is an outcome, never an
        // action error: returning an error would invite a duplicate transaction.
        self.previews.clear();
        recheck(&mut self.checks, &app.workspace);
        self.refresh(app);
        let receipt = RecommendationReceipt {
            key: preview.recommendation.key.clone(),
            command,
            revision: app.workspace.document.revision,
            check_phase: self.checks.phase().clone(),
        };
        self.last_receipt = Some(receipt.clone());
        Ok(receipt)
    }
}
