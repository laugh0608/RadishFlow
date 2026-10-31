use super::*;

#[test]
fn catalog_models_create_with_canonical_identity_and_survive_history_and_storage() {
    for model in rf_unitops::catalog::builtin_models() {
        let mut app = AppState::new(sample_document());
        let created = app
            .create_builtin_unit(model.spec.kind, timestamp(11))
            .unwrap();
        let flowsheet = app.workspace.document.flowsheet.clone();
        let unit = &flowsheet.units[&created.unit_id];
        assert_eq!(unit.kind, model.spec.kind.as_str());
        assert_eq!(rf_unitops::validate_unit_node(unit).unwrap(), model.spec);
        assert!(unit.ports.iter().all(|port| port.stream_id.is_none()));
        assert!(unit.parameters.is_empty());
        app.undo_document_command(timestamp(12)).unwrap().unwrap();
        assert!(app.workspace.document.flowsheet.units.is_empty());
        app.redo_document_command(timestamp(13)).unwrap().unwrap();
        assert_eq!(app.workspace.document.flowsheet, flowsheet);

        let file = rf_store::StoredProjectFile::new(
            flowsheet.clone(),
            rf_store::StoredDocumentMetadata::new("catalog-project", "Catalog", timestamp(10)),
        );
        if model.spec.kind == BuiltinUnitKind::Splitter {
            assert!(rf_store::project_file_to_pretty_json(&file).is_err());
            let project = rf_store::project_v4::ProjectV4Read::Editable(Box::new(
                rf_store::project_v4::StoredProjectV4 {
                    document: file.document,
                    display_units: rf_types::units::DisplayUnitSet::si(),
                    engineering: None,
                    steady_state_plan: rf_store::project_v4::StoredFlowPlan::Acyclic,
                },
            ));
            let json = rf_store::project_v4::to_pretty_json(&project).unwrap();
            assert_eq!(rf_store::project_v4::parse(&json).unwrap(), project);
            continue;
        }
        let json = rf_store::project_file_to_pretty_json(&file).unwrap();
        let reopened = rf_store::parse_project_file_json(&json).unwrap();
        assert_eq!(reopened.document.flowsheet, flowsheet);
    }
}
