use super::*;
use rf_store::{
    FileOverwritePolicy,
    project_v4::{
        self, FlowPlanSupport, PreparedProjectV4Write, ProjectV4Read, RequiredCapability,
        StoredFlowPlan,
    },
};
use rf_ui::{
    WorkspaceState,
    flow_plan::{FlowPlanConfiguration, FlowPlanEditContext},
};
use std::collections::BTreeMap;

/// A capability-aware v4 save review bound to the selected inputs and target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedFlowPlanSave {
    support: FlowPlanSupport,
    source: SaveSource,
    prepared: PreparedProjectV4Write,
    required_capabilities: Vec<RequiredCapability>,
    compatibility: Option<ProjectSaveCompatibility>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SaveSource {
    plan: FlowPlanEditContext,
    path: Option<PathBuf>,
    file_version: Option<u32>,
    saved_revision: Option<u64>,
    saved_capabilities: BTreeMap<String, u32>,
}

impl SaveSource {
    fn capture(workspace: &WorkspaceState) -> Self {
        Self {
            plan: FlowPlanEditContext::capture(workspace),
            path: workspace.document_path.clone(),
            file_version: workspace.project_presentation.source_file_version(),
            saved_revision: workspace.last_saved_revision,
            saved_capabilities: workspace.flow_plan.saved_capabilities().clone(),
        }
    }
}

/// Only a validated mixed-recycle plan can enter the existing Studio save command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedMixedRecycleSave(PreparedFlowPlanSave);

impl PreparedMixedRecycleSave {
    pub fn path(&self) -> &Path {
        self.0.path()
    }
    pub fn compatibility(&self) -> Option<&ProjectSaveCompatibility> {
        self.0.compatibility()
    }
    pub fn replaces_file(&self) -> bool {
        self.0.replaces_file()
    }
    pub fn commit(
        self,
        app: &mut AppState,
        capability_confirmed: bool,
    ) -> RfResult<DocumentLifecycleOutcome> {
        self.0.commit(app, capability_confirmed)
    }
}

pub fn prepare_mixed_recycle_save(
    app: &AppState,
    path: &Path,
) -> RfResult<PreparedMixedRecycleSave> {
    let support = FlowPlanSupport::MixedRecycle;
    let project = flow_plan::project_with_support(app, support)?;
    if !matches!(&project, ProjectV4Read::Editable(p) if matches!(p.steady_state_plan, StoredFlowPlan::MixedRecycle(_)))
    {
        return Err(
            RfError::invalid_input("the applied project plan is not mixed recycle")
                .with_diagnostic_code("workspace.plan.mixed_recycle_required"),
        );
    }
    prepare_plan_save(app, path, support, project).map(PreparedMixedRecycleSave)
}

/// Prepare a save of the applied EO composition specification, including partial configurations.
pub fn prepare_product_composition_save(
    app: &AppState,
    path: &Path,
) -> RfResult<PreparedFlowPlanSave> {
    let support = FlowPlanSupport::ProductComposition;
    let project = flow_plan::project_with_support(app, support)?;
    if !matches!(&project, ProjectV4Read::Editable(p) if p.steady_state_plan.requires_product_composition())
    {
        return Err(RfError::invalid_input(
            "the applied plan does not require product composition support",
        )
        .with_diagnostic_code("workspace.plan.product_composition_required"));
    }
    prepare_plan_save(app, path, support, project)
}

pub(crate) fn prepare_studio_flow_plan_save(
    app: &AppState,
    path: &Path,
) -> RfResult<Option<PreparedFlowPlanSave>> {
    match app.workspace.flow_plan.current() {
        FlowPlanConfiguration::SinglePhase(_) => {
            let support = FlowPlanSupport::SinglePhase;
            let project = flow_plan::project_with_support(app, support)?;
            prepare_plan_save(app, path, support, project).map(Some)
        }
        FlowPlanConfiguration::EquationOriented(options)
            if options.requires_product_composition() =>
        {
            prepare_product_composition_save(app, path).map(Some)
        }
        FlowPlanConfiguration::Mixed(options) if options.recycle.is_some() => {
            prepare_mixed_recycle_save(app, path).map(|p| Some(p.0))
        }
        _ => Ok(None),
    }
}

fn prepare_plan_save(
    app: &AppState,
    path: &Path,
    support: FlowPlanSupport,
    project: ProjectV4Read,
) -> RfResult<PreparedFlowPlanSave> {
    if path.as_os_str().is_empty() {
        return Err(RfError::invalid_input("project save path is empty"));
    }
    let ProjectV4Read::Editable(inputs) = &project else {
        unreachable!("application input is editable");
    };
    let required_capabilities = project_v4::required_capabilities(inputs);
    let prepared = project_v4::prepare_write_with_support(
        path,
        &project,
        FileOverwritePolicy::Allow,
        support,
    )?;
    let compatibility = flow_plan::prepared_compatibility(app, &prepared, &required_capabilities);
    Ok(PreparedFlowPlanSave {
        support,
        source: SaveSource::capture(&app.workspace),
        prepared,
        required_capabilities,
        compatibility,
    })
}

impl PreparedFlowPlanSave {
    pub fn requires_single_phase(&self) -> bool {
        self.support == FlowPlanSupport::SinglePhase
    }
    pub fn requires_product_composition(&self) -> bool {
        self.support == FlowPlanSupport::ProductComposition
    }
    pub fn path(&self) -> &Path {
        self.prepared.path()
    }

    pub fn compatibility(&self) -> Option<&ProjectSaveCompatibility> {
        self.compatibility.as_ref()
    }

    pub fn replaces_file(&self) -> bool {
        self.prepared.target_fingerprint().is_some()
    }

    pub fn commit(
        self,
        app: &mut AppState,
        capability_confirmed: bool,
    ) -> RfResult<DocumentLifecycleOutcome> {
        if self.source != SaveSource::capture(&app.workspace)
            || !self
                .prepared
                .captures_project(&flow_plan::project_with_support(app, self.support)?)?
        {
            return Err(
                RfError::invalid_input("project inputs changed since save preparation")
                    .with_diagnostic_code("workspace.project.save_source_changed"),
            );
        }
        if self.compatibility.is_some() && !capability_confirmed {
            return Err(
                RfError::invalid_input("project v4 upgrade requires confirmation")
                    .with_diagnostic_code("workspace.project.save_confirmation_required"),
            );
        }
        let path = self.prepared.path().to_owned();
        let action = if self.source.path.as_deref() == Some(path.as_path()) {
            StudioDocumentLifecycleAction::Save
        } else {
            StudioDocumentLifecycleAction::SaveAs
        };
        flow_plan::write_prepared_project(&self.prepared, capability_confirmed)?;
        Ok(publish_project_save(
            app,
            action,
            path,
            project_v4::VERSION,
            Some(self.required_capabilities),
        ))
    }
}
