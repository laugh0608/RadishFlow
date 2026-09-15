use std::path::{Path, PathBuf};

use rf_store::{StoredDocumentMetadata, StoredProjectFile, read_project_file, write_project_file};
use rf_types::{RfError, RfResult};
use rf_ui::{AppLogLevel, AppState, DocumentMetadata, FlowsheetDocument};

pub const FILE_SAVE_COMMAND_ID: &str = "file.save";
pub const FILE_SAVE_AS_COMMAND_ID: &str = "file.save_as";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioDocumentLifecycleCommand {
    Save,
    SaveAs { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioDocumentLifecycleAction {
    Save,
    SaveAs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentLifecycleOutcome {
    pub action: StudioDocumentLifecycleAction,
    pub path: PathBuf,
    pub revision: u64,
    pub last_saved_revision: Option<u64>,
    pub has_unsaved_changes: bool,
}

/// Load only project inputs; runtime, cache and window state belong to the caller.
pub fn load_project_app_state(project_path: &Path) -> RfResult<AppState> {
    let stored = read_project_file(project_path)?.document;
    let metadata = stored.metadata;
    let mut document = FlowsheetDocument::new(
        stored.flowsheet,
        DocumentMetadata::new(metadata.document_id, metadata.title, metadata.created_at),
    );
    document.revision = stored.revision;
    document.metadata.schema_version = metadata.schema_version;
    document.metadata.updated_at = metadata.updated_at;

    let mut app_state = AppState::new(document);
    app_state.mark_saved(project_path.to_path_buf());
    Ok(app_state)
}

pub fn dispatch_document_lifecycle(
    app_state: &mut AppState,
    command: StudioDocumentLifecycleCommand,
) -> RfResult<DocumentLifecycleOutcome> {
    let (action, path) = match command {
        StudioDocumentLifecycleCommand::Save => {
            let path =
                app_state.workspace.document_path.clone().ok_or_else(|| {
                    RfError::invalid_input("current document has no project path")
                })?;
            (StudioDocumentLifecycleAction::Save, path)
        }
        StudioDocumentLifecycleCommand::SaveAs { path } => {
            if path.as_os_str().is_empty() {
                return Err(RfError::invalid_input("save-as project path is empty"));
            }
            (StudioDocumentLifecycleAction::SaveAs, path)
        }
    };

    let project_file = stored_project_file_from_app_state(app_state);
    write_project_file(&path, &project_file)?;
    app_state.mark_saved(path.clone());
    app_state.log_feed.push(
        AppLogLevel::Info,
        format!(
            "saved project revision {} to {}",
            app_state.workspace.document.revision,
            path.display()
        ),
    );
    app_state.refresh_run_panel_state();

    Ok(DocumentLifecycleOutcome {
        action,
        path,
        revision: app_state.workspace.document.revision,
        last_saved_revision: app_state.workspace.last_saved_revision,
        has_unsaved_changes: app_state.workspace.last_saved_revision
            != Some(app_state.workspace.document.revision),
    })
}

fn stored_project_file_from_app_state(app_state: &AppState) -> StoredProjectFile {
    let document = &app_state.workspace.document;
    let metadata = &document.metadata;
    let mut project_file = StoredProjectFile::new(
        document.flowsheet.clone(),
        StoredDocumentMetadata {
            document_id: metadata.document_id.as_str().to_string(),
            title: metadata.title.clone(),
            schema_version: metadata.schema_version,
            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
        },
    );
    project_file.document.revision = document.revision;
    project_file
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use rf_store::read_project_file;
    use rf_types::StreamId;

    use super::*;

    fn temp_project_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected time after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("radishflow-{name}-{unique}.rfproj.json"))
    }

    fn app_state_from_example(path: &PathBuf) -> AppState {
        let project_json = crate::test_support::official_heater_binary_hydrocarbon_project_json();
        fs::write(path, project_json).expect("expected temporary project file");
        app_state_from_project_file(path)
    }

    fn app_state_from_project_file(path: &PathBuf) -> AppState {
        let project_file = read_project_file(path).expect("expected project file");
        let metadata = &project_file.document.metadata;
        let mut document = rf_ui::FlowsheetDocument::new(
            project_file.document.flowsheet,
            rf_ui::DocumentMetadata::new(
                metadata.document_id.clone(),
                metadata.title.clone(),
                metadata.created_at,
            ),
        );
        document.revision = project_file.document.revision;
        document.metadata.schema_version = metadata.schema_version;
        document.metadata.updated_at = metadata.updated_at;
        let mut app_state = AppState::new(document);
        app_state.mark_saved(path.clone());
        app_state
    }

    #[test]
    fn variable_write_saves_reopens_with_stable_identity_and_resolves_new_results() {
        use rf_ui::variable_browser::{
            ObjectId, VariableBrowser, VariableField, VariableId, VariableSection, VariableValue,
        };
        use rf_ui::variable_commands::VariableWriteRequest;
        let path = temp_project_path("variable-write");
        let mut app = app_state_from_example(&path);
        let provider = crate::test_support::build_official_binary_hydrocarbon_in_memory_provider(
            crate::test_support::OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
        );
        let service = crate::WorkspaceSolveService::new();
        let package = crate::test_support::OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID;
        service
            .run_with_property_package(&mut app, &provider, package)
            .unwrap();
        let original = rf_ui::latest_snapshot(&app.workspace).unwrap().clone();
        let id = VariableId {
            document: app.workspace.document.metadata.document_id.clone(),
            object: ObjectId::Unit(rf_types::UnitId::new("heater-1")),
            section: VariableSection::Inputs,
            field: VariableField::OutletTemperature,
        };
        app.write_variable(
            VariableWriteRequest {
                variable: id.clone(),
                expected_revision: app.workspace.document.revision,
                value: VariableValue::Number(330.),
            },
            SystemTime::now(),
        )
        .unwrap();
        assert!(rf_ui::latest_snapshot(&app.workspace).is_none());
        assert_eq!(rf_ui::stale_snapshot(&app.workspace), Some(&original));
        dispatch_document_lifecycle(&mut app, StudioDocumentLifecycleCommand::Save).unwrap();
        let bytes = fs::read(&path).unwrap();
        let saved = read_project_file(&path).unwrap();
        assert_eq!(saved.document.revision, app.workspace.document.revision);
        let mut reopened = app_state_from_project_file(&path);
        assert!(rf_ui::latest_snapshot(&reopened.workspace).is_none());
        assert_eq!(
            VariableBrowser::new(&reopened.workspace.document, None, None)
                .read(&id)
                .unwrap()
                .value,
            Some(VariableValue::Number(330.))
        );
        service
            .run_with_property_package(&mut reopened, &provider, package)
            .unwrap();
        let result = rf_ui::latest_snapshot(&reopened.workspace).unwrap();
        assert_eq!(result.document_revision, saved.document.revision);
        let heater = result
            .steps
            .iter()
            .find(|s| s.unit_id.as_str() == "heater-1")
            .unwrap();
        assert_eq!(heater.streams[0].temperature_k, 330.);
        let flash = result
            .steps
            .iter()
            .find(|s| s.unit_id.as_str() == "flash-1")
            .unwrap();
        assert_eq!(heater.streams, flash.consumed_streams);
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert_eq!(
            reopened.workspace.last_saved_revision,
            Some(result.document_revision)
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn save_current_project_writes_document_and_marks_revision_saved() {
        let path = temp_project_path("save-current");
        let mut app_state = app_state_from_example(&path);
        let mut next_flowsheet = app_state.workspace.document.flowsheet.clone();
        next_flowsheet
            .streams
            .get_mut(&StreamId::new("stream-feed"))
            .expect("expected feed stream")
            .temperature_k = 333.5;
        app_state.commit_document_change(
            rf_ui::DocumentCommand::SetStreamSpecification {
                stream_id: StreamId::new("stream-feed"),
                field: "temperature_k".to_string(),
                value: rf_ui::CommandValue::Number(333.5),
            },
            next_flowsheet,
            SystemTime::now(),
        );

        let outcome =
            dispatch_document_lifecycle(&mut app_state, StudioDocumentLifecycleCommand::Save)
                .expect("expected save outcome");

        assert_eq!(outcome.action, StudioDocumentLifecycleAction::Save);
        assert_eq!(outcome.path, path);
        assert_eq!(outcome.last_saved_revision, Some(outcome.revision));
        assert!(!outcome.has_unsaved_changes);
        assert_eq!(
            app_state.workspace.last_saved_revision,
            Some(app_state.workspace.document.revision)
        );

        let saved = read_project_file(&outcome.path).expect("expected saved project");
        assert_eq!(saved.document.revision, outcome.revision);
        assert_eq!(
            saved
                .document
                .flowsheet
                .streams
                .get(&StreamId::new("stream-feed"))
                .expect("expected saved feed stream")
                .temperature_k,
            333.5
        );
    }

    #[test]
    fn save_as_writes_new_path_and_updates_workspace_path() {
        let source_path = temp_project_path("save-as-source");
        let target_path = temp_project_path("save-as-target");
        let mut app_state = app_state_from_example(&source_path);

        let outcome = dispatch_document_lifecycle(
            &mut app_state,
            StudioDocumentLifecycleCommand::SaveAs {
                path: target_path.clone(),
            },
        )
        .expect("expected save-as outcome");

        assert_eq!(outcome.action, StudioDocumentLifecycleAction::SaveAs);
        assert_eq!(outcome.path, target_path);
        assert_eq!(
            app_state.workspace.document_path.as_deref(),
            Some(outcome.path.as_path())
        );
        assert_eq!(
            app_state.workspace.last_saved_revision,
            Some(outcome.revision)
        );
        assert!(read_project_file(&outcome.path).is_ok());
    }

    #[test]
    fn save_as_failure_keeps_workspace_path_saved_revision_and_history() {
        let source_path = temp_project_path("save-as-failure-source");
        let target_path = temp_project_path("save-as-failure-target");
        fs::create_dir_all(&target_path).expect("expected directory target");
        let mut app_state = app_state_from_example(&source_path);
        let original_saved_revision = app_state.workspace.last_saved_revision;
        let mut next_flowsheet = app_state.workspace.document.flowsheet.clone();
        next_flowsheet
            .streams
            .get_mut(&StreamId::new("stream-feed"))
            .expect("expected feed stream")
            .pressure_pa = 123_456.0;
        let dirty_revision = app_state.commit_document_change(
            rf_ui::DocumentCommand::SetStreamSpecification {
                stream_id: StreamId::new("stream-feed"),
                field: "pressure_pa".to_string(),
                value: rf_ui::CommandValue::Number(123_456.0),
            },
            next_flowsheet,
            SystemTime::now(),
        );

        let error = dispatch_document_lifecycle(
            &mut app_state,
            StudioDocumentLifecycleCommand::SaveAs {
                path: target_path.clone(),
            },
        )
        .expect_err("expected save-as failure");

        assert!(
            error
                .message()
                .contains("target path exists and is not a file")
        );
        assert_eq!(
            app_state.workspace.document_path.as_deref(),
            Some(source_path.as_path())
        );
        assert_eq!(
            app_state.workspace.last_saved_revision,
            original_saved_revision
        );
        assert_eq!(app_state.workspace.document.revision, dirty_revision);
        assert_ne!(
            app_state.workspace.last_saved_revision,
            Some(app_state.workspace.document.revision)
        );
        assert_eq!(app_state.workspace.command_history.len(), 1);
        assert!(target_path.is_dir());

        let _ = fs::remove_dir_all(target_path);
        let _ = fs::remove_file(source_path);
    }
}
