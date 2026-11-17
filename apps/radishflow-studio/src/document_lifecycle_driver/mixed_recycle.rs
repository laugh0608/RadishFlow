use super::*;
use rf_store::{
    FileOverwritePolicy,
    project_v4::{
        self, FlowPlanSupport, PreparedProjectV4Write, ProjectV4Read, RequiredCapability,
        StoredFlowPlan,
    },
};
use rf_ui::{WorkspaceState, flow_plan::FlowPlanEditContext};
use std::collections::BTreeMap;

/// A v4 mixed-recycle save review bound to the selected inputs and target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedMixedRecycleSave {
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

/// Explicit entry for a caller that supports the mixed-recycle project capability.
pub fn prepare_mixed_recycle_save(
    app: &AppState,
    path: &Path,
) -> RfResult<PreparedMixedRecycleSave> {
    if path.as_os_str().is_empty() {
        return Err(RfError::invalid_input("mixed recycle save path is empty"));
    }
    let project = flow_plan::project_with_support(app, FlowPlanSupport::MixedRecycle)?;
    let ProjectV4Read::Editable(inputs) = &project else {
        unreachable!("application input is editable");
    };
    if !matches!(inputs.steady_state_plan, StoredFlowPlan::MixedRecycle(_)) {
        return Err(
            RfError::invalid_input("the applied project plan is not mixed recycle")
                .with_diagnostic_code("workspace.plan.mixed_recycle_required"),
        );
    }
    let required_capabilities = project_v4::required_capabilities(inputs);
    let prepared = project_v4::prepare_write_with_support(
        path,
        &project,
        FileOverwritePolicy::Allow,
        FlowPlanSupport::MixedRecycle,
    )?;
    let compatibility = flow_plan::prepared_compatibility(app, &prepared, &required_capabilities);
    Ok(PreparedMixedRecycleSave {
        source: SaveSource::capture(&app.workspace),
        prepared,
        required_capabilities,
        compatibility,
    })
}

impl PreparedMixedRecycleSave {
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
                .captures_project(&flow_plan::project_with_support(
                    app,
                    FlowPlanSupport::MixedRecycle,
                )?)?
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
