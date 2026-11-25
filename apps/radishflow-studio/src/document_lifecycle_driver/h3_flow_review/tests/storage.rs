use super::*;
use rf_store::FileOverwritePolicy;

fn target(fixture: &Fixture) -> PathBuf {
    fixture.path.with_file_name("linked.rfproj.json")
}

fn json_file(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn prepare(review: &H3FlowReview, target: &Path) -> PreparedH3FlowAssociationSave {
    review
        .prepare_association_save(preview(review).unwrap(), target, FileOverwritePolicy::Allow)
        .unwrap()
}

fn save_new(fixture: &Fixture) -> PathBuf {
    let target = target(fixture);
    let review = fixture.solved();
    let prepared = prepare(&review, &target);
    assert!(!prepared.requires_upgrade());
    review.save_association(&prepared, false).unwrap();
    target
}

#[test]
fn h3_flow_association_edit_replaces_the_cold_stream_in_the_same_project() {
    let fixture = Fixture::new();
    let alternate = "cold-feed-50";
    fixture.amend_project(|root| {
        let flowsheet = &mut root["document"]["flowsheet"];
        let mut cold = flowsheet["streams"][COLD].clone();
        cold["id"] = json!(alternate);
        cold["total_molar_flow_mol_s"] = json!(50.0);
        flowsheet["streams"][alternate] = cold;
        let mut feed = flowsheet["units"]["cold-feed-unit"].clone();
        feed["id"] = json!("cold-feed-unit-50");
        feed["ports"][0]["stream_id"] = json!(alternate);
        flowsheet["units"]["cold-feed-unit-50"] = feed;
        root["objectIdentities"]["streams"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "objectId":alternate,"creationId":"00000000-0000-4000-8000-000000000050"
            }));
        root["objectIdentities"]["units"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "objectId":"cold-feed-unit-50","creationId":"00000000-0000-4000-8000-000000000051"
            }));
    });

    let review = fixture.solved();
    let pair = |review: &H3FlowReview, cold| ConcentricTubeFlowAssociation {
        hot: stream(review, HOT),
        cold: stream(review, cold),
    };
    let first = review
        .preview_association_edit(member(), pair(&review, COLD))
        .unwrap();
    let save = review
        .prepare_association_edit_save(&first, H3AssociationSaveTarget::Source)
        .unwrap();
    assert!(save.requires_upgrade());
    assert!(review.save_association(&save, false).is_err());
    review.save_association(&save, true).unwrap();

    let mut opened = fixture.open();
    assert_eq!(
        opened.workspace().flow_result().status(),
        ResultStatus::Missing
    );
    opened.solve_flow(&fixture.provider).unwrap();
    let old_rating = opened
        .rate(opened.preview_saved(member()).unwrap())
        .unwrap();
    assert_benchmark(opened.current_rating(&old_rating).unwrap(), 40);
    assert_code(
        opened
            .preview_association_edit(member(), pair(&opened, COLD))
            .unwrap_err(),
        "association_unchanged",
    );
    assert_code(
        opened
            .preview(member(), stream(&opened, HOT), stream(&opened, alternate))
            .unwrap_err(),
        "association_mismatch",
    );

    let candidate = opened
        .preview_association_edit(member(), pair(&opened, alternate))
        .unwrap();
    let original = fs::read(&fixture.path).unwrap();
    let copy = target(&fixture);
    let prepared_copy = opened
        .prepare_association_edit_save(&candidate, H3AssociationSaveTarget::NewFile(&copy))
        .unwrap();
    fs::write(&copy, b"external writer").unwrap();
    let conflict = opened.save_association(&prepared_copy, false).unwrap_err();
    assert_eq!(
        conflict.context().diagnostic_code(),
        Some("store.project.save_target_changed")
    );
    assert_eq!(fs::read(&copy).unwrap(), b"external writer");
    assert_eq!(fs::read(&fixture.path).unwrap(), original);

    let prepared = opened
        .prepare_association_edit_save(&candidate, H3AssociationSaveTarget::Source)
        .unwrap();
    assert!(!prepared.requires_upgrade());
    opened.save_association(&prepared, false).unwrap();
    assert_code(
        opened.save_association(&prepared, false).unwrap_err(),
        "source_changed",
    );
    let mut reopened = fixture.open();
    assert_eq!(
        reopened.workspace().flow_result().status(),
        ResultStatus::Missing
    );
    assert_code(
        reopened.current_rating(&old_rating).unwrap_err(),
        "inputs_changed",
    );
    reopened.solve_flow(&fixture.provider).unwrap();
    let next_rating = reopened
        .rate(reopened.preview_saved(member()).unwrap())
        .unwrap();
    assert_benchmark(reopened.current_rating(&next_rating).unwrap(), 50);

    let old: Value = serde_json::from_slice(&original).unwrap();
    let new = json_file(&fixture.path);
    let old_task = &old["engineering"]["tasks"][0];
    let new_task = &new["engineering"]["tasks"][0];
    assert_eq!(new_task["geometry"], old_task["geometry"]);
    assert_eq!(
        new_task["revision"].as_u64(),
        old_task["revision"].as_u64().map(|rev| rev + 1)
    );
    assert_eq!(
        new_task["case"]["revision"].as_u64(),
        old_task["case"]["revision"].as_u64().map(|rev| rev + 1)
    );
    assert_eq!(new["document"]["flowsheet"], old["document"]["flowsheet"]);
    assert_eq!(new["objectIdentities"], old["objectIdentities"]);
}

#[test]
fn h3_flow_storage_reopens_both_pure_water_loads_with_only_formal_references() {
    for cold_flow in [40, 50] {
        let fixture = Fixture::new();
        fixture.amend_project(|root| {
            root["document"]["flowsheet"]["streams"][COLD]["total_molar_flow_mol_s"] =
                json!(f64::from(cold_flow));
            let reference: Value = serde_json::from_str(include_str!(
                "../../../../tests/fixtures/h3-review-engineering.json"
            ))
            .unwrap();
            root["engineering"]["tasks"]
                .as_array_mut()
                .unwrap()
                .push(reference["tasks"][1].clone());
        });
        let initial_bytes = fs::read(&fixture.path).unwrap();
        let independent = fixture.path.with_file_name("independent.rfproj.json");
        fs::write(&independent, &initial_bytes).unwrap();
        let initial = json_file(&fixture.path);
        let target = if cold_flow == 40 {
            fixture.path.clone()
        } else {
            target(&fixture)
        };
        let review = fixture.solved();
        let previous = review.rate(preview(&review).unwrap()).unwrap();
        let previous_preview = preview(&review).unwrap();
        let hot = stream(&review, HOT);
        let cold = stream(&review, COLD);
        let prepared = prepare(&review, &target);
        assert_eq!(prepared.path(), target);
        assert_eq!(prepared.requires_upgrade(), cold_flow == 40);
        if prepared.requires_upgrade() {
            let error = review.save_association(&prepared, false).unwrap_err();
            assert!(
                error.message().contains("upgrade requires confirmation"),
                "{error}"
            );
            assert_eq!(fs::read(&target).unwrap(), initial_bytes);
        }
        review
            .save_association(&prepared, prepared.requires_upgrade())
            .unwrap();
        if target != fixture.path {
            assert_eq!(fs::read(&fixture.path).unwrap(), initial_bytes);
        }
        let linked = json_file(&target);
        let task = &linked["engineering"]["tasks"][0];
        assert_eq!(
            task["case"]["flowAssociation"],
            json!({
                "hot":{"streamId":HOT,"creationId":hot.creation_id.as_str()},
                "cold":{"streamId":COLD,"creationId":cold.creation_id.as_str()},
            })
        );
        assert!(task["case"]["hot"].get("inlet").is_none());
        assert!(task["case"]["cold"].get("inlet").is_none());
        assert_eq!(task["method"], initial["engineering"]["tasks"][0]["method"]);
        assert_eq!(
            task["geometry"],
            initial["engineering"]["tasks"][0]["geometry"]
        );
        assert_eq!(task["revision"].as_u64(), Some(6));
        assert_eq!(task["case"]["revision"].as_u64(), Some(12));
        assert_eq!(linked["document"]["revision"].as_u64(), Some(1));
        assert_eq!(
            linked["document"]["flowsheet"],
            initial["document"]["flowsheet"]
        );
        assert_eq!(linked["objectIdentities"], initial["objectIdentities"]);
        assert_eq!(
            linked["engineering"]["sources"],
            initial["engineering"]["sources"]
        );
        assert_eq!(
            linked["engineering"]["tasks"][1],
            initial["engineering"]["tasks"][1]
        );
        let linked_bytes = fs::read(&target).unwrap();

        let mut reopened = H3FlowReview::open(&target, fixture.qualification.clone()).unwrap();
        assert_eq!(
            reopened.workspace().flow_result().status(),
            ResultStatus::Missing
        );
        assert_code(
            reopened.preview_saved(member()).unwrap_err(),
            "flow_unavailable",
        );
        assert_code(
            reopened.current_rating(&previous).unwrap_err(),
            "inputs_changed",
        );
        assert_code(
            reopened.rate(previous_preview).unwrap_err(),
            "inputs_changed",
        );
        let saved = reopened.saved.case(&member()).unwrap();
        assert_eq!(saved.case.flow_association.as_ref().unwrap().hot, hot);
        assert_eq!(saved.case.flow_association.as_ref().unwrap().cold, cold);
        let Err(SavedRatingError::Calculation(error)) =
            reopened.saved.rate(&member(), saved.into()).unwrap()
        else {
            panic!("an independent calculation cannot use a saved association");
        };
        assert_code(error, "association_unresolved");
        reopened.solve_flow(&fixture.provider).unwrap();
        assert_code(
            reopened.preview(member(), cold, hot).unwrap_err(),
            "association_mismatch",
        );
        let result = reopened
            .rate(reopened.preview_saved(member()).unwrap())
            .unwrap();
        assert_benchmark(reopened.current_rating(&result).unwrap(), cold_flow);
        assert_eq!(fs::read(&target).unwrap(), linked_bytes);

        // The explicitly supplied binary is built from the frozen old source.
        if let Some(binary) = std::env::var_os("RADISHFLOW_H3_FLOW_LEGACY_CHECK") {
            let output = std::process::Command::new(binary)
                .args([&target, &independent])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            println!("{}", String::from_utf8_lossy(&output.stdout));
            assert_eq!(fs::read(&target).unwrap(), linked_bytes);
        }
    }
}

#[test]
fn h3_flow_storage_rejects_changed_source_target_and_preview_without_writing() {
    let fixture = Fixture::new();
    let original = fs::read(&fixture.path).unwrap();
    let mut review = fixture.solved();
    let target = target(&fixture);
    fs::write(&target, &original).unwrap();
    let prepared = prepare(&review, &target);
    let mut changed = original.clone();
    changed.push(b'\n');
    fs::write(&fixture.path, &changed).unwrap();
    assert_code(
        review.save_association(&prepared, true).unwrap_err(),
        "source_changed",
    );
    assert_eq!(fs::read(&fixture.path).unwrap(), changed);
    assert_eq!(fs::read(&target).unwrap(), original);

    fs::write(&fixture.path, &original).unwrap();
    fs::write(&target, &changed).unwrap();
    let error = review.save_association(&prepared, true).unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("store.project.save_target_changed")
    );
    assert_eq!(fs::read(&target).unwrap(), changed);
    assert_eq!(fs::read(&fixture.path).unwrap(), original);

    let prepared = prepare(&review, &target);
    review.solve_flow(&fixture.provider).unwrap();
    assert_code(
        review.save_association(&prepared, true).unwrap_err(),
        "inputs_changed",
    );
    edit_flow(&mut review, 50.0);
    review.solve_flow(&fixture.provider).unwrap();
    assert_code(
        review
            .prepare_association_save(
                preview(&review).unwrap(),
                &target,
                FileOverwritePolicy::Allow,
            )
            .unwrap_err(),
        "source_changed",
    );
    assert_eq!(fs::read(&target).unwrap(), changed);
    assert_eq!(fs::read(&fixture.path).unwrap(), original);
}

#[test]
fn h3_flow_storage_write_failure_leaves_the_saved_inputs_and_result_intact() {
    let fixture = Fixture::new();
    let original = fs::read(&fixture.path).unwrap();
    let review = fixture.solved();
    let rating = review.rate(preview(&review).unwrap()).unwrap();
    let target = fixture
        .path
        .with_file_name("absent-parent")
        .join("linked.rfproj.json");
    let prepared = prepare(&review, &target);
    assert!(review.save_association(&prepared, false).is_err());
    assert!(!target.exists());
    assert_eq!(fs::read(&fixture.path).unwrap(), original);
    assert_benchmark(review.current_rating(&rating).unwrap(), 40);
    assert!(
        review
            .saved
            .case(&member())
            .unwrap()
            .case
            .flow_association
            .is_none()
    );
}

#[test]
fn h3_flow_storage_validates_references_exclusive_inlets_and_capability() {
    let fixture = Fixture::new();
    let target = save_new(&fixture);
    let original_bytes = fs::read(&target).unwrap();
    let document = json_file(&target);
    let reference = "/engineering/tasks/0/case/flowAssociation";
    let capability = project_v4::CONCENTRIC_TUBE_FLOW_ASSOCIATION_CAPABILITY;
    let mut invalid = document.clone();
    invalid.pointer_mut(reference).unwrap()["cold"] =
        document.pointer(reference).unwrap()["hot"].clone();
    assert_parse_error(
        &invalid,
        ".cold.streamId: hot and cold streams must be distinct",
    );
    let mut invalid = document.clone();
    invalid.pointer_mut(reference).unwrap()["cold"]["streamId"] = json!("missing");
    assert_parse_error(&invalid, ".cold.streamId: stream must exist");
    let mut invalid = document.clone();
    invalid.pointer_mut(reference).unwrap()["cold"]["creationId"] =
        document.pointer(reference).unwrap()["hot"]["creationId"].clone();
    assert_parse_error(&invalid, ".cold.creationId: does not match");
    let mut invalid = document.clone();
    invalid.pointer_mut(reference).unwrap()["cold"] = json!({"streamId": COLD});
    assert_parse_error(&invalid, "missing field `creationId`");
    let mut invalid = document.clone();
    invalid.pointer_mut(reference).unwrap()["cold"]["documentId"] = json!("other");
    assert_parse_error(&invalid, "unknown field `documentId`");
    let mut invalid = document.clone();
    *invalid.pointer_mut(reference).unwrap() = Value::Null;
    assert_parse_error(&invalid, "expected a JSON object");
    let mut invalid = document.clone();
    invalid["engineering"]["tasks"][0]["case"]["cold"]["inlet"] = json!({"molarFlowMolS":40.0});
    assert_parse_error(
        &invalid,
        "case.cold.inlet: a flow-associated case cannot store inlet numbers",
    );
    let mut invalid = document.clone();
    invalid["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .retain(|entry| entry["id"] != capability);
    assert_parse_error(
        &invalid,
        "requiredCapabilities: capability declarations do not match",
    );
    let mut invalid = document.clone();
    invalid.as_object_mut().unwrap().remove("objectIdentities");
    assert_parse_error(&invalid, "requires saved objectIdentities");
    let mut future = document;
    future["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["id"] == capability)
        .unwrap()["version"] = json!(2);
    let future_bytes = serde_json::to_string(&future).unwrap();
    let ProjectV4Read::ReadOnly(future) =
        project_v4::parse_with_equipment_support(&future_bytes, FLOW_SUPPORT, EQUIPMENT_SUPPORT)
            .unwrap()
    else {
        panic!("future association capability must stay opaque");
    };
    assert_eq!(future.original_json(), future_bytes);
    assert_eq!(fs::read(&target).unwrap(), original_bytes);
}

fn assert_parse_error(document: &Value, reason: &str) {
    let error = parse_project(&serde_json::to_string(document).unwrap()).unwrap_err();
    assert!(
        error.message().contains(reason),
        "expected {reason:?}, got {error}"
    );
}

#[test]
fn h3_flow_storage_rejects_downgrade_and_preserves_unaware_readers() {
    let fixture = Fixture::new();
    let target = save_new(&fixture);
    let original = fs::read_to_string(&target).unwrap();
    let unsupported = project_v4::read_with_equipment_support(
        &target,
        FLOW_SUPPORT,
        EquipmentSupport::ConcentricTubeRating,
    )
    .unwrap();
    let ProjectV4Read::ReadOnly(view) = &unsupported else {
        panic!("independent H3 reader must keep linked projects opaque");
    };
    assert_eq!(view.original_json(), original);
    let independent = ProjectV4Read::Editable(
        parse_project(&fs::read_to_string(&fixture.path).unwrap()).unwrap(),
    );
    assert!(
        project_v4::prepare_write_with_equipment_support(
            &target,
            &independent,
            FileOverwritePolicy::Allow,
            FLOW_SUPPORT,
            EquipmentSupport::ConcentricTubeRating,
        )
        .is_err()
    );
    let error = project_v4::prepare_write_with_equipment_support(
        &target,
        &independent,
        FileOverwritePolicy::Allow,
        FLOW_SUPPORT,
        EQUIPMENT_SUPPORT,
    )
    .unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("store.project.h3_flow_association_removal")
    );
    let linked = parse_project(&original).unwrap();
    let v3 = rf_store::project_v3::ProjectV3Read::Editable(rf_store::project_v3::StoredProjectV3 {
        document: linked.document.clone(),
        display_units: linked.display_units.clone(),
        engineering: linked.engineering.clone(),
    });
    let error = rf_store::project_v3::to_pretty_json_with_equipment_support(&v3, EQUIPMENT_SUPPORT)
        .unwrap_err();
    assert!(error.message().contains("flow associations require v4"));
    let mut downgraded: Value = serde_json::from_str(&original).unwrap();
    for key in ["simulation", "requiredCapabilities", "objectIdentities"] {
        downgraded.as_object_mut().unwrap().remove(key);
    }
    downgraded["schemaVersion"] = json!(3);
    let error = rf_store::project_v3::parse_with_equipment_support(
        &serde_json::to_string(&downgraded).unwrap(),
        EQUIPMENT_SUPPORT,
    )
    .unwrap_err();
    assert!(error.message().contains("flow associations require v4"));
    assert_eq!(fs::read_to_string(&target).unwrap(), original);
}
