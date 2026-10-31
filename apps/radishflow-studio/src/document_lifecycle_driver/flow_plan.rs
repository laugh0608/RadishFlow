use super::*;
use crate::flow_plan_actions::{flow_plan_configuration, stored_flow_plan};
use rf_store::project_v4::{self, ProjectV4Read, StoredProjectV4};

#[derive(Debug)]
pub enum StudioReadOnlyProject {
    V3(rf_store::project_v3::ReadOnlyProjectV3),
    V4(project_v4::ReadOnlyProjectV4),
}
impl StudioReadOnlyProject {
    pub fn original_json(&self) -> &str {
        match self {
            Self::V3(p) => p.original_json(),
            Self::V4(p) => p.original_json(),
        }
    }
    pub fn metadata(&self) -> &rf_store::StoredDocumentMetadata {
        match self {
            Self::V3(p) => &p.document().metadata,
            Self::V4(p) => p.metadata(),
        }
    }
    pub fn revision(&self) -> u64 {
        match self {
            Self::V3(p) => p.document().revision,
            Self::V4(p) => p.revision(),
        }
    }
    pub fn schema_version(&self) -> u32 {
        match self {
            Self::V3(_) => 3,
            Self::V4(_) => 4,
        }
    }
    pub fn task_count(&self) -> Option<usize> {
        match self {
            Self::V3(p) => Some(p.task_count()),
            Self::V4(_) => None,
        }
    }
    pub fn legacy_document(&self) -> Option<&rf_store::StoredProjectDocument> {
        match self {
            Self::V3(p) => Some(p.document()),
            Self::V4(_) => None,
        }
    }
    pub fn reason_summary(&self) -> String {
        match self {
            Self::V3(p) => format!(
                "{} equipment tasks; unsupported method or legacy payload",
                p.task_count()
            ),
            Self::V4(p) => p.reasons().join(", "),
        }
    }
}
pub(super) fn parse_v4(contents: &str, path: &Path) -> RfResult<StudioProjectOpen> {
    match project_v4::parse(contents)? {
        ProjectV4Read::ReadOnly(p) => Ok(StudioProjectOpen::ReadOnly(Box::new(
            StudioReadOnlyProject::V4(p),
        ))),
        ProjectV4Read::Editable(p) => {
            let configuration = flow_plan_configuration(p.steady_state_plan)?;
            let mut app =
                app_state_from_stored(p.document, p.display_units, 4, p.engineering, path)?;
            app.workspace.flow_plan = rf_ui::flow_plan::FlowPlanState::from_loaded(configuration);
            app.workspace.reserve_flow_plan_identities();
            Ok(StudioProjectOpen::Editable(Box::new(app)))
        }
    }
}
pub(super) fn project(app: &AppState) -> ProjectV4Read {
    let file = stored_project_file_from_app_state(app);
    let saved = app.workspace.engineering.capture_save();
    ProjectV4Read::Editable(Box::new(StoredProjectV4 {
        document: file.document,
        display_units: file.presentation.display_units,
        engineering: saved
            .block_present()
            .then(|| rf_store::project_v3::StoredEngineeringInputs {
                basis: saved.inputs().basis.clone(),
                tasks: saved.inputs().tasks.clone(),
            }),
        steady_state_plan: stored_flow_plan(app.workspace.flow_plan.current()),
    }))
}
pub(super) fn compatibility(
    app: &AppState,
    path: &Path,
) -> RfResult<Option<ProjectSaveCompatibility>> {
    let prepared =
        project_v4::prepare_write(path, &project(app), rf_store::FileOverwritePolicy::Allow)?;
    let source = app.workspace.project_presentation.source_file_version();
    Ok(
        (source != Some(4) || prepared.requires_upgrade()).then_some(
            ProjectSaveCompatibility::Format {
                version: 4,
                includes_rating: app.workspace.engineering.inputs().has_rating(),
            },
        ),
    )
}
pub(super) fn save(app: &AppState, path: &Path, upgrade_confirmed: bool) -> RfResult<()> {
    let prepared =
        project_v4::prepare_write(path, &project(app), rf_store::FileOverwritePolicy::Allow)?;
    if app.workspace.project_presentation.source_file_version() != Some(4) && !upgrade_confirmed {
        return Err(RfError::invalid_input(
            "project v4 upgrade requires confirmation",
        ));
    }
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|e| RfError::invalid_input(format!("create project directory: {e}")))?;
    }
    project_v4::write_prepared(&prepared, upgrade_confirmed)
}
#[cfg(test)]
mod tests;
