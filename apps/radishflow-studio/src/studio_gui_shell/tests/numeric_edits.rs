use super::*;

#[test]
fn si_inspector_keeps_explicit_input_units_until_native_unit_controls_are_connected() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    let before = app.platform_host.snapshot();
    let key = "stream:stream-feed:pressure_pa";
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id(key),
        "1.2 bar",
    );
    let invalid = app.platform_host.snapshot();
    let field = invalid
        .runtime
        .active_inspector_detail
        .as_ref()
        .unwrap()
        .property_fields
        .iter()
        .find(|field| field.key == key)
        .unwrap();
    assert!(field.commit_command_id.is_none());
    assert_eq!(field.current_value, "1.2 bar");
    assert_eq!(field.label, "Pressure (Pa)");
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id(key),
        "110000 Pa",
    );
    let edited = app.platform_host.snapshot();
    let field = edited
        .runtime
        .active_inspector_detail
        .as_ref()
        .unwrap()
        .property_fields
        .iter()
        .find(|field| field.key == key)
        .unwrap();
    assert_eq!(field.label, "Pressure (Pa)");
    assert_eq!(field.current_value, "110000 Pa");
    assert_eq!(
        edited.runtime.workspace_document.revision,
        before.runtime.workspace_document.revision
    );
    assert_eq!(
        edited.runtime.latest_solve_snapshot,
        before.runtime.latest_solve_snapshot
    );
    app.dispatch_inspector_field_draft_commit(field.commit_command_id.as_ref().unwrap().clone());
    assert_eq!(
        app.platform_host.document().flowsheet.streams[&StreamId::new("stream-feed")].pressure_pa,
        110000.0
    );
    let committed = app.platform_host.snapshot();
    assert_eq!(
        committed
            .runtime
            .workspace_document
            .save_state
            .pending_input_count,
        0
    );
    assert!(committed.runtime.latest_solve_snapshot.is_none());
    app.dispatch_ui_command("edit.undo");
    assert_eq!(
        app.platform_host.document().flowsheet.streams[&StreamId::new("stream-feed")].pressure_pa,
        120000.0
    );
}
