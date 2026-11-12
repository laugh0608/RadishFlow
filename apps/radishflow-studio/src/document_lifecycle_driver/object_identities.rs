use super::*;
use rf_store::{
    FileOverwritePolicy,
    project_v4::{self, PreparedProjectV4Write, ProjectV4Read, RequiredCapability},
};
use rf_types::FlowObjectIdentities;
use rf_ui::{
    InputEditCheckpoint, ProjectPresentationState, WorkspaceState,
    engineering::EngineeringCheckpoint,
    flow_plan::{FlowPlanEditContext, FlowPlanGeneration},
    project_query::{ProjectInputContext, ProjectQuery},
};

/// The exact document and target reviewed for first-time identity initialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedObjectIdentityInitialization {
    source: InitializationCheckpoint,
    write: PreparedProjectV4Write,
    allocated: FlowObjectIdentities,
    capabilities: Vec<RequiredCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InitializationCheckpoint {
    input: ProjectInputContext,
    drafts: InputEditCheckpoint,
    plan: FlowPlanEditContext,
    plan_draft: FlowPlanGeneration,
    equipment_draft: EngineeringCheckpoint,
    presentation: ProjectPresentationState,
    source_path: Option<PathBuf>,
    saved_revision: Option<u64>,
    saved_capabilities: std::collections::BTreeMap<String, u32>,
}

impl InitializationCheckpoint {
    fn capture(workspace: &WorkspaceState) -> Self {
        Self {
            input: ProjectQuery::new(workspace).input_context(),
            drafts: InputEditCheckpoint::capture(
                workspace.document.metadata.document_id.as_str(),
                workspace.document.revision,
                &workspace.drafts.fields,
            ),
            plan: FlowPlanEditContext::capture(workspace),
            plan_draft: workspace.flow_plan.draft_generation().clone(),
            equipment_draft: workspace.equipment_editor.checkpoint(),
            presentation: workspace.project_presentation.clone(),
            source_path: workspace.document_path.clone(),
            saved_revision: workspace.last_saved_revision,
            saved_capabilities: workspace.flow_plan.saved_capabilities().clone(),
        }
    }
}

pub fn prepare_object_identity_initialization(
    app: &AppState,
    path: &Path,
) -> RfResult<PreparedObjectIdentityInitialization> {
    if path.as_os_str().is_empty() {
        return Err(RfError::invalid_input(
            "object identity initialization requires a project path",
        ));
    }
    let workspace = &app.workspace;
    let allocated = workspace.prepare_object_identities()?;
    let ProjectV4Read::Editable(mut project) = flow_plan::project(app) else {
        unreachable!("the active workspace is editable")
    };
    project.object_identities = Some(WorkspaceState::project_object_identities(
        &allocated,
        &project.document.flowsheet,
    ));
    let capabilities = project_v4::required_capabilities(&project);
    let write = project_v4::prepare_write(
        path,
        &ProjectV4Read::Editable(project),
        FileOverwritePolicy::Allow,
    )?;
    if write.target_has_object_identities() {
        return Err(RfError::invalid_input(
            "target already has object identities; choose another path",
        ));
    }
    Ok(PreparedObjectIdentityInitialization {
        source: InitializationCheckpoint::capture(workspace),
        write,
        allocated,
        capabilities,
    })
}

impl PreparedObjectIdentityInitialization {
    pub fn path(&self) -> &Path {
        self.write.path()
    }

    pub fn replaces_file(&self) -> bool {
        self.write.target_fingerprint().is_some()
    }

    pub fn source_version(&self) -> Option<u32> {
        self.source.presentation.source_file_version()
    }

    pub(super) fn commit(self, app: &mut AppState) -> RfResult<DocumentLifecycleOutcome> {
        if app.workspace.object_identities_enabled()
            || self.source != InitializationCheckpoint::capture(&app.workspace)
        {
            return Err(RfError::invalid_input(
                "project or input changed after object identity initialization was prepared",
            )
            .with_diagnostic_code("project.object_identity.stale_initialization"));
        }
        let path = self.write.path().to_owned();
        let action = if self.source.source_path.as_deref() == Some(&path) {
            StudioDocumentLifecycleAction::Save
        } else {
            StudioDocumentLifecycleAction::SaveAs
        };
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|error| {
                RfError::invalid_input(format!("create project directory: {error}"))
            })?;
        }
        project_v4::write_prepared(&self.write, true)?;
        app.workspace
            .install_saved_object_identities(self.allocated);
        Ok(publish_project_save(
            app,
            action,
            path,
            project_v4::VERSION,
            Some(self.capabilities),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document_lifecycle_driver::tests::temp_project_path;
    use rf_ui::{BuiltinUnitKind, variable_browser::ObjectId};
    use std::{fs, time::UNIX_EPOCH};

    fn source_project(path: &Path) -> AppState {
        fs::write(
            path,
            include_str!(
                "../../../../examples/flowsheets/feed-heater-flash-synthetic-demo.rfproj.json"
            ),
        )
        .unwrap();
        load_project_app_state(path).unwrap()
    }

    #[test]
    fn initialization_publishes_saved_identities_and_preserves_existing_history() {
        let source = temp_project_path("identity-source");
        let target = temp_project_path("identity-target");
        let mut app = source_project(&source);
        let original = fs::read(&source).unwrap();
        let removed = app
            .create_builtin_unit(BuiltinUnitKind::Heater, UNIX_EPOCH)
            .unwrap()
            .unit_id;
        app.delete_unit(&removed, UNIX_EPOCH).unwrap();
        let history = app.workspace.command_history.clone();
        let revision = app.workspace.document.revision;
        let prepared = prepare_object_identity_initialization(&app, &target).unwrap();
        assert!(!app.workspace.object_identities_enabled());
        prepared.commit(&mut app).unwrap();
        assert_eq!(app.workspace.command_history, history);
        assert_eq!(app.workspace.document.revision, revision);
        assert_eq!(app.workspace.document_path.as_ref(), Some(&target));
        assert_eq!(fs::read(&source).unwrap(), original);
        let reopened = load_project_app_state(&target).unwrap();
        assert!(reopened.workspace.object_identities_enabled());
        assert_eq!(
            reopened.workspace.current_object_identities(),
            app.workspace.current_object_identities()
        );
        assert!(prepare_object_identity_initialization(&app, &target).is_err());
        let mut old = load_project_app_state(&source).unwrap();
        assert!(prepare_object_identity_initialization(&old, &target).is_err());
        assert!(
            dispatch_document_lifecycle(
                &mut old,
                StudioDocumentLifecycleCommand::SaveAs {
                    path: target.clone()
                }
            )
            .is_err()
        );

        app.undo_document_command(UNIX_EPOCH).unwrap().unwrap();
        let object = ObjectId::Unit(removed);
        let restored = app.workspace.object_creation_id(&object).unwrap().clone();
        assert!(app.workspace.saved_object_creation_id(&object).is_none());
        dispatch_document_lifecycle(&mut app, StudioDocumentLifecycleCommand::Save).unwrap();
        assert_eq!(
            app.workspace.saved_object_creation_id(&object),
            Some(&restored)
        );
        let mut reopened = load_project_app_state(&target).unwrap();
        assert_eq!(
            reopened.workspace.object_creation_id(&object),
            Some(&restored)
        );
        let ObjectId::Unit(removed) = object else {
            unreachable!()
        };
        reopened.delete_unit(&removed, UNIX_EPOCH).unwrap();
        dispatch_document_lifecycle(&mut reopened, StudioDocumentLifecycleCommand::Save).unwrap();
        let mut reopened = load_project_app_state(&target).unwrap();
        let recreated = reopened
            .create_builtin_unit(BuiltinUnitKind::Heater, UNIX_EPOCH)
            .unwrap()
            .unit_id;
        assert_eq!(recreated, removed);
        let object = ObjectId::Unit(recreated);
        assert_ne!(
            reopened.workspace.object_creation_id(&object),
            Some(&restored)
        );
        assert!(
            reopened
                .workspace
                .saved_object_creation_id(&object)
                .is_none()
        );
        fs::remove_file(source).unwrap();
        fs::remove_file(target).unwrap();
    }

    #[test]
    fn stale_or_failed_initialization_never_publishes_identity_or_switches_path() {
        let source = temp_project_path("identity-stale-source");
        let target = temp_project_path("identity-stale-target");
        let mut app = source_project(&source);
        let original = fs::read(&source).unwrap();
        let stale = prepare_object_identity_initialization(&app, &target).unwrap();
        app.create_builtin_unit(BuiltinUnitKind::Heater, UNIX_EPOCH)
            .unwrap();
        assert!(stale.commit(&mut app).is_err());
        assert!(!target.exists());

        let prepared = prepare_object_identity_initialization(&app, &target).unwrap();
        fs::write(&target, "concurrent writer").unwrap();
        let unchanged = app.workspace.clone();
        assert!(prepared.commit(&mut app).is_err());
        assert_eq!(app.workspace, unchanged);
        assert_eq!(fs::read(&source).unwrap(), original);
        assert_eq!(fs::read_to_string(&target).unwrap(), "concurrent writer");
        fs::remove_file(&target).unwrap();

        let unavailable = target.join("project.rfproj.json");
        let prepared = prepare_object_identity_initialization(&app, &unavailable).unwrap();
        fs::write(&target, "not a directory").unwrap();
        assert!(prepared.commit(&mut app).is_err());
        assert_eq!(app.workspace, unchanged);
        fs::remove_file(source).unwrap();
        fs::remove_file(target).unwrap();
    }

    #[test]
    fn initializes_in_place_from_all_supported_envelopes_without_changing_formal_inputs() {
        let v2 =
            include_str!("../../../../crates/rf-store/tests/fixtures/project-v2-engineering.json");
        let mut v1: serde_json::Value = serde_json::from_str(v2).unwrap();
        v1["schemaVersion"] = 1.into();
        v1.as_object_mut().unwrap().remove("presentation");
        let v1 = v1.to_string();
        let v3 = include_str!(
            "../../../../crates/rf-store/tests/fixtures/project-v3-rating-adoption.json"
        );
        let v4 = include_str!(
            "../../../../examples/flowsheets/fixed-tp-recycle-binary-hydrocarbon.rfproj.json"
        );
        for (version, text) in [(1, v1.as_str()), (2, v2), (3, v3), (4, v4)] {
            let path = temp_project_path("identity-in-place");
            fs::write(&path, text).unwrap();
            let mut app = load_project_app_state(&path).unwrap();
            assert_eq!(
                app.workspace.project_presentation.source_file_version(),
                Some(version)
            );
            let before = app.workspace.clone();
            let prepared = prepare_object_identity_initialization(&app, &path).unwrap();
            assert_eq!(fs::read_to_string(&path).unwrap(), text);
            assert_eq!(
                prepared.commit(&mut app).unwrap().action,
                StudioDocumentLifecycleAction::Save
            );
            let after = load_project_app_state(&path).unwrap().workspace;
            assert_eq!(after.document, before.document);
            assert_eq!(after.engineering.inputs(), before.engineering.inputs());
            assert_eq!(after.flow_plan.current(), before.flow_plan.current());
            assert_eq!(
                after.project_presentation.display_units(),
                before.project_presentation.display_units()
            );
            assert!(after.object_identities_enabled());
            fs::remove_file(path).unwrap();
        }
    }
}
