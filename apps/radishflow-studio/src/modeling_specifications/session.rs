use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelingCheckFailure {
    Panicked,
    RejectedReportIdentity,
    Check(RfError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelingCheckPhase {
    NotChecked,
    Stale,
    Checking,
    Current,
    Failed(ModelingCheckFailure),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelingCheckTicket {
    pub identity: SpecificationReportIdentity,
    pub generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecificationEntryState {
    Current,
    Unconfirmed,
    TargetRemoved,
    Resolved,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpecificationEntry {
    pub key: SpecificationProblemKey,
    pub problem: SpecificationProblem,
    pub observed_revision: u64,
    pub state: SpecificationEntryState,
}

/// One owner per open workspace. This is ephemeral state, never project storage.
/// Replacing the workspace advances its session even for the same persisted ID.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelingCheckSession {
    plan_generation: Option<rf_ui::flow_plan::FlowPlanGeneration>,
    identity: SpecificationReportIdentity,
    generation: u64,
    phase: ModelingCheckPhase,
    report: Option<ModelingSpecificationReport>,
    entries: Vec<SpecificationEntry>,
    changes: Vec<SpecificationEntry>,
}

impl ModelingCheckSession {
    pub fn new(document: &FlowsheetDocument) -> Self {
        Self {
            plan_generation: None,
            identity: SpecificationReportIdentity {
                document_id: document.metadata.document_id.clone(),
                session: ModelingCheckSessionId(1),
                input_revision: document.revision,
            },
            generation: 0,
            phase: ModelingCheckPhase::NotChecked,
            report: None,
            entries: Vec::new(),
            changes: Vec::new(),
        }
    }

    pub fn replace_document(&mut self, document: &FlowsheetDocument) {
        let session = ModelingCheckSessionId(
            self.identity
                .session
                .0
                .checked_add(1)
                .expect("check session exhausted"),
        );
        *self = Self::new(document);
        self.identity.session = session;
    }

    pub fn identity(&self) -> &SpecificationReportIdentity {
        &self.identity
    }
    pub fn plan_generation(&self) -> Option<&rf_ui::flow_plan::FlowPlanGeneration> {
        self.plan_generation.as_ref()
    }
    pub fn phase(&self) -> &ModelingCheckPhase {
        &self.phase
    }
    pub fn report(&self) -> Option<&ModelingSpecificationReport> {
        self.report.as_ref()
    }
    pub fn current_report(&self) -> Option<&ModelingSpecificationReport> {
        self.report
            .as_ref()
            .filter(|_| self.phase == ModelingCheckPhase::Current)
    }
    pub fn entries(&self) -> &[SpecificationEntry] {
        &self.entries
    }
    pub fn changes(&self) -> &[SpecificationEntry] {
        &self.changes
    }

    pub fn current_count(&self) -> Option<usize> {
        (self.phase == ModelingCheckPhase::Current).then(|| {
            self.entries
                .iter()
                .filter(|e| e.state == SpecificationEntryState::Current)
                .count()
        })
    }

    /// Call after formal edits, before publishing any UI projection. Raw drafts
    /// and presentation changes do not advance the document input revision.
    pub fn observe_document(&mut self, document: &FlowsheetDocument) {
        if self.identity.document_id != document.metadata.document_id {
            self.replace_document(document);
        } else if self.identity.input_revision != document.revision {
            self.identity.input_revision = document.revision;
            self.phase = ModelingCheckPhase::Stale;
        }
    }

    pub fn begin(&mut self, document: &FlowsheetDocument) -> ModelingCheckTicket {
        self.observe_document(document);
        self.generation = self
            .generation
            .checked_add(1)
            .expect("check generation exhausted");
        self.phase = ModelingCheckPhase::Checking;
        ModelingCheckTicket {
            identity: self.identity.clone(),
            generation: self.generation,
        }
    }

    /// A late result cannot mutate a newer revision, check, or document session.
    pub fn finish(
        &mut self,
        ticket: &ModelingCheckTicket,
        result: Result<ModelingSpecificationReport, ModelingCheckFailure>,
        document: &FlowsheetDocument,
    ) -> bool {
        self.observe_document(document);
        if ticket.identity != self.identity
            || ticket.generation != self.generation
            || self.phase != ModelingCheckPhase::Checking
        {
            return false;
        }
        let report = match result {
            Ok(report)
                if report.identity == ticket.identity
                    && report.contract_version == SPECIFICATION_CONTRACT_VERSION =>
            {
                report
            }
            Ok(_) => {
                self.phase =
                    ModelingCheckPhase::Failed(ModelingCheckFailure::RejectedReportIdentity);
                return true;
            }
            Err(error) => {
                self.phase = ModelingCheckPhase::Failed(error);
                return true;
            }
        };
        let mut next: Vec<SpecificationEntry> = Vec::new();
        for problem in &report.problems {
            let key = problem.key(&report.identity);
            if !next.iter().any(|entry| entry.key == key) {
                next.push(SpecificationEntry {
                    key,
                    problem: problem.clone(),
                    observed_revision: report.identity.input_revision,
                    state: SpecificationEntryState::Current,
                });
            }
        }
        self.changes.clear();
        for previous in &self.entries {
            if next.iter().any(|entry| entry.key == previous.key) {
                continue;
            }
            let mut previous = previous.clone();
            if previous
                .key
                .targets
                .iter()
                .any(|target| !target.exists(document))
            {
                previous.state = SpecificationEntryState::TargetRemoved;
                self.changes.push(previous);
            } else if report.coverage.iter().any(|coverage| {
                coverage.scope == previous.key.scope
                    && coverage.coverage == SpecificationCoverage::Evaluated
            }) {
                previous.state = SpecificationEntryState::Resolved;
                self.changes.push(previous);
            } else {
                previous.state = SpecificationEntryState::Unconfirmed;
                next.push(previous);
            }
        }
        self.entries = next;
        self.report = Some(report);
        self.phase = ModelingCheckPhase::Current;
        true
    }

    pub fn recheck(&mut self, document: &FlowsheetDocument) {
        let ticket = self.begin(document);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            check_modeling_specifications(document, ticket.identity.session)
        }))
        .map_err(|_| ModelingCheckFailure::Panicked);
        self.finish(&ticket, result, document);
    }

    pub fn refresh(&mut self, document: &FlowsheetDocument) {
        self.observe_document(document);
        if matches!(
            self.phase,
            ModelingCheckPhase::NotChecked | ModelingCheckPhase::Stale
        ) {
            self.recheck(document);
        }
    }
}

impl ModelingCheckSession {
    pub fn refresh_workspace(&mut self, workspace: &rf_ui::WorkspaceState) {
        let changed = self.plan_generation.as_ref() != Some(workspace.flow_plan.generation());
        self.observe_document(&workspace.document);
        if changed
            || matches!(
                self.phase,
                ModelingCheckPhase::NotChecked | ModelingCheckPhase::Stale
            )
        {
            self.recheck_workspace(workspace);
        }
    }
    pub fn begin_workspace(
        &mut self,
        workspace: &rf_ui::WorkspaceState,
    ) -> WorkspaceModelingCheckTicket {
        let ticket = self.begin(&workspace.document);
        WorkspaceModelingCheckTicket {
            ticket,
            plan: rf_ui::flow_plan::FlowPlanEditContext::capture(workspace),
        }
    }
    pub fn finish_workspace(
        &mut self,
        ticket: &WorkspaceModelingCheckTicket,
        result: Result<ModelingSpecificationReport, ModelingCheckFailure>,
        workspace: &rf_ui::WorkspaceState,
    ) -> bool {
        if !ticket.plan.is_current(workspace) {
            self.phase = ModelingCheckPhase::Stale;
            return false;
        }
        let result = result.and_then(|report| {
            if report.plan_generation.as_ref() != Some(workspace.flow_plan.generation()) {
                Err(ModelingCheckFailure::RejectedReportIdentity)
            } else {
                Ok(report)
            }
        });
        let accepted = self.finish(&ticket.ticket, result, &workspace.document);
        if accepted {
            self.plan_generation = Some(workspace.flow_plan.generation().clone());
        }
        accepted
    }
    pub fn recheck_workspace(&mut self, workspace: &rf_ui::WorkspaceState) {
        let ticket = self.begin_workspace(workspace);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            check_workspace_modeling_specifications(workspace, ticket.ticket.identity.session)
        }))
        .map_err(|_| ModelingCheckFailure::Panicked);
        self.finish_workspace(&ticket, result, workspace);
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceModelingCheckTicket {
    ticket: ModelingCheckTicket,
    plan: rf_ui::flow_plan::FlowPlanEditContext,
}
