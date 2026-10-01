use super::*;
use rf_types::units::DisplayUnitSet;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

const V1: &str =
    include_str!("../../../../examples/flowsheets/feed-heater-flash-synthetic-demo.rfproj.json");
const V2: &str =
    include_str!("../../../../crates/rf-store/tests/fixtures/project-v2-engineering.json");

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "rf-presentation-lifecycle-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn legacy_open_and_save_do_not_silently_upgrade_or_dirty_the_document() {
    let f = Fixture::new();
    let path = f.path("old.rfproj.json");
    fs::write(&path, V1).unwrap();
    let mut app = load_project_app_state(&path).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), V1);
    assert_eq!(
        app.workspace.project_presentation.display_units(),
        &DisplayUnitSet::si()
    );
    assert_eq!(
        app.workspace.project_presentation.source_file_version(),
        Some(1)
    );
    assert_eq!(
        app.workspace.last_saved_revision,
        Some(app.workspace.document.revision)
    );
    let before = app.workspace.document.clone();
    let state = app.clone();
    assert!(dispatch_document_lifecycle(&mut app, StudioDocumentLifecycleCommand::Save).is_err());
    assert_eq!(app, state);
    assert_eq!(fs::read_to_string(&path).unwrap(), V1);
    let outcome = dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() },
    )
    .unwrap();
    assert!(!outcome.has_unsaved_changes);
    assert_eq!(app.workspace.document, before);
    assert_eq!(read_project_file(&path).unwrap().schema_version, 2);
}

#[test]
fn v2_save_as_and_reopen_preserve_presentation_without_physical_changes() {
    let f = Fixture::new();
    let original = f.path("v2.rfproj.json");
    let copy = f.path("copy.rfproj.json");
    fs::write(&original, V2).unwrap();
    let mut app = load_project_app_state(&original).unwrap();
    let before = app.workspace.document.clone();
    let solve = app.workspace.solve_session.clone();
    assert_eq!(
        app.workspace.project_presentation.display_units(),
        &DisplayUnitSet::engineering()
    );
    assert_eq!(
        app.workspace.project_presentation.source_file_version(),
        Some(2)
    );
    let outcome = dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::SaveAs { path: copy.clone() },
    )
    .unwrap();
    assert!(!outcome.has_unsaved_changes);
    assert_eq!(app.workspace.document, before);
    assert_eq!(app.workspace.solve_session, solve);
    assert!(app.workspace.command_history.is_empty());
    assert_eq!(fs::read_to_string(&original).unwrap(), V2);
    let reopened = load_project_app_state(&copy).unwrap();
    assert_eq!(reopened.workspace.document, before);
    assert_eq!(
        reopened.workspace.project_presentation,
        app.workspace.project_presentation
    );
    assert_eq!(read_project_file(&copy).unwrap().schema_version, 2);
}

#[test]
fn v2_failed_save_preserves_loaded_configuration_and_saved_path() {
    let f = Fixture::new();
    let path = f.path("v2.rfproj.json");
    fs::write(&path, V2).unwrap();
    let mut app = load_project_app_state(&path).unwrap();
    let before = app.clone();
    let directory_target = f.path("directory");
    fs::create_dir(&directory_target).unwrap();
    assert!(
        dispatch_document_lifecycle(
            &mut app,
            StudioDocumentLifecycleCommand::SaveAs {
                path: directory_target
            }
        )
        .is_err()
    );
    assert_eq!(app, before);
    assert_eq!(fs::read_to_string(&path).unwrap(), V2);
}

#[test]
fn new_gui_document_writes_v2_without_legacy_upgrade_prompt() {
    let f = Fixture::new();
    let mut app = AppState::new(FlowsheetDocument::new(
        rf_model::Flowsheet::new("new"),
        DocumentMetadata::new("new-doc", "New", SystemTime::now()),
    ));
    let path = f.path("new.rfproj.json");
    assert_eq!(
        app.workspace.project_presentation.source_file_version(),
        None
    );
    dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::SaveAs { path: path.clone() },
    )
    .unwrap();
    assert_eq!(read_project_file(path).unwrap().schema_version, 2);
    assert_eq!(
        app.workspace.project_presentation.source_file_version(),
        Some(2)
    );
}

#[test]
fn presentation_save_failure_and_undo_keep_physics_results_and_drafts_separate() {
    use rf_ui::ProjectPresentationCommand;
    let f = Fixture::new();
    let path = f.path("v2.rfproj.json");
    fs::write(&path, V2).unwrap();
    let mut app = load_project_app_state(&path).unwrap();
    let before = app.workspace.document.clone();
    let solve = app.workspace.solve_session.clone();
    app.workspace
        .project_presentation
        .apply(ProjectPresentationCommand::Apply(DisplayUnitSet::si()));
    let save = app.workspace.project_save_state();
    assert!(!save.document_dirty);
    assert!(save.presentation_dirty);
    assert!(save.needs_close_confirmation());
    let bad = f.path("directory");
    fs::create_dir(&bad).unwrap();
    let pending = app.clone();
    assert!(
        dispatch_document_lifecycle(
            &mut app,
            StudioDocumentLifecycleCommand::SaveAs { path: bad }
        )
        .is_err()
    );
    assert_eq!(app, pending);
    let outcome =
        dispatch_document_lifecycle(&mut app, StudioDocumentLifecycleCommand::Save).unwrap();
    assert!(!outcome.save_state.has_unsaved_changes());
    assert_eq!(app.workspace.document, before);
    assert_eq!(app.workspace.solve_session, solve);
    assert!(app.workspace.command_history.is_empty());
    assert_eq!(
        load_project_app_state(&path)
            .unwrap()
            .workspace
            .project_presentation
            .display_units(),
        &DisplayUnitSet::si()
    );
    assert!(
        app.workspace
            .project_presentation
            .apply(ProjectPresentationCommand::Undo)
    );
    assert!(app.workspace.project_save_state().presentation_dirty);
    assert_eq!(app.workspace.document, before);
    assert!(
        app.workspace
            .project_presentation
            .apply(ProjectPresentationCommand::Redo)
    );
    assert!(!app.workspace.project_save_state().has_unsaved_changes());
}
