use super::*;
use rf_solver::partitioned::{FixedRecycleBoundary, RecycleSolveOptions};
use rf_ui::flow_plan::*;
use std::fs;
const LEGACY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../crates/rf-store/tests/fixtures/project-v2-engineering.json"
));
fn app() -> AppState {
    let StudioProjectOpen::Editable(app) =
        parse_studio_project(LEGACY, Path::new("input.rfproj.json")).unwrap()
    else {
        unreachable!()
    };
    *app
}
fn select_partitioned(app: &mut AppState) {
    let plan = FlowPlanConfiguration::Partitioned(RecycleSolveOptions::new(
        FixedRecycleBoundary {
            temperature_k: 300.0,
            pressure_pa: 650_000.0,
        },
        vec![],
    ));
    app.workspace
        .apply_flow_plan(
            &FlowPlanEditContext::capture(&app.workspace),
            FlowPlanCommand::Apply(plan),
        )
        .unwrap();
}
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "rf-plan-lifecycle-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn old_project_stays_clean_and_explicit_plan_requires_an_upgrade() {
    let mut app = app();
    assert_eq!(project_save_version(&app), 2);
    assert!(!app.workspace.project_save_state().has_unsaved_changes());
    let revision = app.workspace.document.revision;
    select_partitioned(&mut app);
    assert_eq!(app.workspace.document.revision, revision);
    assert_eq!(project_save_version(&app), 4);
    let dir = Directory::new();
    let path = dir.0.join("upgraded.json");
    fs::write(&path, LEGACY).unwrap();
    assert!(
        dispatch_document_lifecycle(
            &mut app,
            StudioDocumentLifecycleCommand::SaveAs { path: path.clone() }
        )
        .is_err()
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), LEGACY);
    assert!(app.workspace.flow_plan.is_dirty());
    dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() },
    )
    .unwrap();
    assert!(!app.workspace.project_save_state().has_unsaved_changes());
    let reopened = load_project_app_state(&path).unwrap();
    assert_eq!(
        reopened.workspace.flow_plan.current(),
        app.workspace.flow_plan.current()
    );
    assert!(
        !reopened
            .workspace
            .project_save_state()
            .has_unsaved_changes()
    );
    assert_eq!(project_save_version(&reopened), 4);
    assert!(matches!(
        reopened.workspace.flow_result(),
        rf_ui::result_query::FlowResultView::Missing { .. }
    ));
    let mut reopened = reopened;
    reopened
        .workspace
        .apply_flow_plan(
            &FlowPlanEditContext::capture(&reopened.workspace),
            FlowPlanCommand::Apply(FlowPlanConfiguration::Acyclic),
        )
        .unwrap();
    assert_eq!(project_save_version(&reopened), 4);
}
#[test]
fn save_keeps_unapplied_draft_and_failed_save_keeps_every_baseline() {
    let mut app = app();
    select_partitioned(&mut app);
    let mut draft = FlowPlanDraft::new(&app.workspace, app.workspace.flow_plan.current().clone());
    draft
        .edit_number(
            FlowPlanNumericField::Pressure,
            650_000.0,
            rf_types::units::MeasurementUnit::Pascal,
        )
        .unwrap()
        .raw = "1e-".into();
    app.workspace.flow_plan.set_draft(draft.clone());
    let dir = Directory::new();
    let path = dir.0.join("plan.json");
    dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() },
    )
    .unwrap();
    assert_eq!(app.workspace.flow_plan.draft(), Some(&draft));
    assert_eq!(app.workspace.flow_plan.pending_count(), 1);
    app.workspace
        .apply_flow_plan(
            &FlowPlanEditContext::capture(&app.workspace),
            FlowPlanCommand::Apply(FlowPlanConfiguration::Acyclic),
        )
        .unwrap();
    let before = app.workspace.clone();
    let bad = dir.0.join("directory");
    fs::create_dir(&bad).unwrap();
    assert!(
        dispatch_document_lifecycle(
            &mut app,
            StudioDocumentLifecycleCommand::SaveAs { path: bad }
        )
        .is_err()
    );
    assert_eq!(app.workspace, before);
}
#[test]
fn unknown_v4_capability_never_creates_an_editable_workspace_and_v3_cli_rejects_v4() {
    let mut app = app();
    select_partitioned(&mut app);
    let text = project_v4::to_pretty_json(&project(&app)).unwrap();
    let unknown = text.replacen("\"version\": 1", "\"version\": 99", 1);
    let StudioProjectOpen::ReadOnly(read) =
        parse_studio_project(&unknown, Path::new("unknown.json")).unwrap()
    else {
        panic!("unknown capabilities must be readonly")
    };
    assert_eq!(read.schema_version(), 4);
    assert_eq!(read.original_json(), unknown);
    assert!(read.legacy_document().is_none());
    let dir = Directory::new();
    let path = dir.0.join("project.json");
    fs::write(&path, text).unwrap();
    let request = dir.0.join("request.json");
    fs::write(
        &request,
        serde_json::json!({"schema_version":3,"project":path,"queries":[]}).to_string(),
    )
    .unwrap();
    let mut stdout = vec![];
    let mut stderr = vec![];
    let status = crate::headless::run_cli(
        &["query".into(), request.into_os_string()],
        &mut stdout,
        &mut stderr,
    );
    assert_ne!(status, 0);
    let response: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(response["error"]["code"], "unsupported_capability");
}
