use super::*;
use radishflow_studio::{flow_actions::FlowEditOutcome, run_actions::StudioRunReply};
use rf_ui::engineering::editor::*;

#[test]
fn gui_equipment_consumes_formal_and_check_receipts_then_reports_busy_without_navigation() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.equipment_command(EquipmentEditorCommand::CreateTask);
    let receipt = app.equipment.last_action.clone().unwrap();
    assert_eq!(receipt.outcome, EquipmentActionOutcome::Committed);
    let Some(EquipmentEditorTarget::Task(task)) = receipt.target else {
        panic!()
    };
    app.equipment_command(EquipmentEditorCommand::Check(task.clone()));
    assert!(
        matches!(&app.equipment.last_action.as_ref().unwrap().outcome, EquipmentActionOutcome::SizingChecked { task: checked, issues } if checked == &task && !issues.is_empty())
    );
    app.enable_background_runs();
    app.dispatch_ui_command("run_panel.run_manual");
    let occupied = app
        .background_runs
        .as_ref()
        .unwrap()
        .borrow()
        .activity()
        .unwrap()
        .id;
    let active = app
        .platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .active()
        .cloned();
    app.equipment_command(EquipmentEditorCommand::Calculate(task));
    assert_eq!(
        app.equipment.last_run_reply,
        Some(StudioRunReply::Busy(occupied))
    );
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor
            .active(),
        active.as_ref()
    );
    app.background_action(radishflow_studio::StudioBackgroundRunAction::BeginShutdown);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while app.background_busy() {
        assert!(std::time::Instant::now() < deadline);
        app.poll_background_runs(&egui::Context::default());
        std::thread::yield_now();
    }
}

#[test]
fn readonly_call_boundary_rejects_injected_equipment_and_flow_events() {
    let path = super::equipment_storage::v3_file("formal-readonly", true);
    let mut app = ready_app_state(&synced_workspace_config());
    app.open_project(path.clone(), "formal read-only test");
    assert!(app.read_only_project.is_some());
    let before = app.platform_host.modeling_inputs().clone();
    let window_id = app.current_window_id().unwrap();
    for event in [
        StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: StudioRuntimeTrigger::Equipment(EquipmentEditorCommand::CreateSource),
        },
        StudioGuiEvent::UiCommandRequested {
            command_id: "run_panel.run_manual".into(),
        },
        StudioGuiEvent::CanvasPendingEditCommitRequested {
            position: rf_ui::CanvasPoint::new(10.0, 20.0),
        },
    ] {
        assert!(app.dispatch_event_result(event).is_err());
        assert_eq!(app.platform_host.modeling_inputs(), &before);
    }
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn gui_numeric_commit_returns_input_context_and_does_not_turn_same_value_into_history() {
    use rf_ui::{
        NumericEditCommand as Command,
        variable_browser::{ObjectId, VariableField, VariableId, VariableSection},
    };
    let mut app = ready_app_state(&synced_workspace_config());
    let variable = VariableId {
        document: app.platform_host.document().metadata.document_id.clone(),
        object: ObjectId::Stream("stream-feed".into()),
        section: VariableSection::Inputs,
        field: VariableField::Pressure,
    };
    let window_id = app.current_window_id().unwrap();
    let dispatch = app
        .dispatch_event_result(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: StudioRuntimeTrigger::NumericEdit(Command::Begin(variable.clone())),
        })
        .unwrap();
    let Some(radishflow_studio::StudioRuntimeDispatch::NumericEdit {
        generation: Some(generation),
        ..
    }) = failure_recovery::runtime_dispatch_from_gui(&dispatch)
    else {
        panic!()
    };
    let generation = *generation;
    let revision = app.platform_host.document().revision;
    let dispatch = app
        .dispatch_event_result(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: StudioRuntimeTrigger::NumericEdit(Command::Commit {
                variable: variable.clone(),
                generation,
            }),
        })
        .unwrap();
    let Some(radishflow_studio::StudioRuntimeDispatch::NumericEdit {
        committed: Some(receipt),
        ..
    }) = failure_recovery::runtime_dispatch_from_gui(&dispatch)
    else {
        panic!()
    };
    assert_eq!(receipt.variable, variable);
    assert_eq!(receipt.outcome, FlowEditOutcome::Unchanged);
    assert_eq!(receipt.before, receipt.after);
    assert_eq!(app.platform_host.document().revision, revision);
}
