use super::*;
use crate::UnitInspectorDraftField;

#[test]
fn rename_unit_matrix_preserves_identity_parameters_connections_and_history() {
    for kind in [
        BuiltinUnitKind::Feed,
        BuiltinUnitKind::Heater,
        BuiltinUnitKind::Cooler,
        BuiltinUnitKind::Valve,
        BuiltinUnitKind::Mixer,
        BuiltinUnitKind::FlashDrum,
    ] {
        let mut app = AppState::new(sample_document());
        app.begin_canvas_place_unit(kind.as_str());
        let id = app
            .commit_canvas_pending_edit_at(CanvasPoint::new(20.0, 30.0), timestamp(1))
            .unwrap()
            .unwrap()
            .unit_id;
        let before = app.workspace.document.flowsheet.clone();
        let revision = app.workspace.document.revision;
        let history = app.workspace.command_history.len();
        let name = "  主设备 A 中文  ";
        app.update_unit_inspector_draft(&id, UnitInspectorDraftField::Name, name)
            .unwrap();
        let result = app
            .commit_unit_inspector_draft(&id, UnitInspectorDraftField::Name, timestamp(2))
            .unwrap()
            .unwrap();
        assert_eq!(result.revision, revision + 1);
        assert_eq!(
            result.command,
            DocumentCommand::RenameUnit {
                unit_id: id.clone(),
                new_name: name.into()
            }
        );
        let mut expected = before.clone();
        expected.units.get_mut(&id).unwrap().name = name.into();
        assert_eq!(app.workspace.document.flowsheet, expected);
        assert_eq!(app.workspace.command_history.len(), history + 1);
        assert!(
            app.commit_unit_inspector_draft(&id, UnitInspectorDraftField::Name, timestamp(3))
                .unwrap()
                .is_none()
        );
        app.undo_document_command(timestamp(4)).unwrap().unwrap();
        assert_eq!(app.workspace.document.flowsheet, before);
        app.redo_document_command(timestamp(5)).unwrap().unwrap();
        assert_eq!(app.workspace.document.flowsheet, expected);
    }
}

#[test]
fn rename_unit_rejects_blank_and_stale_edits_and_allows_duplicate_display_names() {
    let mut document = sample_document();
    document
        .flowsheet
        .insert_unit(UnitNode::new("other", "重复名称", "heater", vec![]))
        .unwrap();
    let mut app = AppState::new(document);
    app.begin_canvas_place_unit("feed");
    let id = app
        .commit_canvas_pending_edit_at(CanvasPoint::new(20.0, 30.0), timestamp(1))
        .unwrap()
        .unwrap()
        .unit_id;
    let original = app.workspace.document.clone();
    for value in ["", " \t\n", original.flowsheet.units[&id].name.as_str()] {
        app.update_unit_inspector_draft(&id, UnitInspectorDraftField::Name, value)
            .unwrap();
        assert!(
            app.commit_unit_inspector_draft(&id, UnitInspectorDraftField::Name, timestamp(2))
                .unwrap()
                .is_none()
        );
        assert_eq!(app.workspace.document, original);
    }
    app.update_unit_inspector_draft(&id, UnitInspectorDraftField::Name, "discard me");
    app.discard_unit_inspector_draft(&id, UnitInspectorDraftField::Name)
        .unwrap();
    assert_eq!(app.workspace.document, original);
    let duplicate = original.flowsheet.units[&UnitId::new("other")].name.clone();
    app.update_unit_inspector_draft(&id, UnitInspectorDraftField::Name, duplicate.clone());
    app.commit_unit_inspector_draft(&id, UnitInspectorDraftField::Name, timestamp(3))
        .unwrap();
    assert_eq!(app.workspace.document.flowsheet.units[&id].name, duplicate);
    app.delete_unit(&id, timestamp(4)).unwrap();
    let deleted = app.clone();
    assert!(
        app.update_unit_inspector_draft(&id, UnitInspectorDraftField::Name, "stale")
            .is_none()
    );
    assert!(
        app.commit_unit_inspector_draft(&id, UnitInspectorDraftField::Name, timestamp(5))
            .unwrap()
            .is_none()
    );
    assert_eq!(app, deleted);
}
