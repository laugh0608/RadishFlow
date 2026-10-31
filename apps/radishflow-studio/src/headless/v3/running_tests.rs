use super::*;
fn adoption_project() -> Value {
    serde_json::from_str(include_str!(
        "../../../../../crates/rf-store/tests/fixtures/project-v3-rating-adoption.json"
    ))
    .unwrap()
}
fn results_query(id: &str, task: Value, case: Value) -> Value {
    json!({"id":id,"query":{"kind":"results","scope":{"object":{"case":{"task":task,"case":case}}}}})
}
fn sizing_step() -> Value {
    json!({"id":"size","action":{"kind":"size","task":{"id":"sizing-source"},"expected_revision":1}})
}
fn adopt_step() -> Value {
    json!({"id":"adopt","action":{"kind":"adopt_geometry","source":{"id":"sizing-source"},"expected_revision":1,"target":{"kind":"new","tag":"V-CLI","name":"CLI adopted separator","copy_case":true},"adjustments":{}}})
}
#[test]
fn sizing_adoption_rating_replays_current_candidate_without_flow_cache() {
    let f = Fixture::new();
    let mut request=f.execution_request(&adoption_project(),json!([
        sizing_step(),adopt_step(),
        {"id":"rate","action":{"kind":"rate","task":reference("adopt","task"),"case":reference("adopt","case"),"expected_case_revision":reference("adopt","case"),"expected_geometry_revision":reference("adopt","geometry")}}
    ]));
    request["queries"] = json!([
        results_query(
            "sizing",
            json!({"id":"sizing-source"}),
            json!({"id":"sizing-source-case"})
        ),
        results_query(
            "rating",
            reference("adopt", "task"),
            reference("adopt", "case")
        )
    ]);
    let (code, first) = f.execute(&request);
    assert_eq!(code, 6, "{first}");
    assert_eq!(first["steps"][0]["terminal"], "succeeded");
    assert_eq!(first["queries"][0]["value"]["status"], "current");
    let preview = &first["error"]["details"]["preview"];
    assert!(
        preview["candidate"]["dimensions"]["minimum_diameter"]["value_si"]
            .as_f64()
            .unwrap()
            > 0.0
    );
    assert!(
        !preview["candidate"]["capture"]["sources"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let digest = first["error"]["details"]["confirmation_digest"].clone();
    let (_, second) = f.execute(&request);
    assert_eq!(second["error"]["details"]["confirmation_digest"], digest);
    request["steps"][1]["confirmation"] = json!({"digest":digest});
    let (code, r) = f.execute(&request);
    assert_eq!(code, 0, "{r}");
    assert_eq!(r["steps"][2]["terminal"], "succeeded");
    assert_eq!(r["steps"][2]["reception"], "applied");
    assert_eq!(
        r["queries"][0]["value"]["origin"]["execution"]["step"],
        "size"
    );
    assert_eq!(
        r["queries"][1]["value"]["origin"]["execution"]["step"],
        "rate"
    );
    assert_eq!(r["queries"][1]["value"]["status"], "current");
    assert!(
        !r["steps"][1]["created_sources"]["copies"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    request["steps"][1]["action"]["adjustments"] =
        json!({"diameter":{"value_m":4.0,"reason":"Explicit clearance adjustment"}});
    let (code, r) = f.execute(&request);
    assert_eq!(code, 6, "{r}");
    assert_eq!(r["error"]["code"], "confirmation_mismatch");
    assert_eq!(r["queries"][0]["value"]["status"], "current");
    assert!(r["queries"][1].get("error").is_some());
}
#[test]
fn sizing_input_change_revokes_candidate_and_stale_query_omits_values() {
    let f = Fixture::new();
    let mut request = f.execution_request(&adoption_project(), json!([]));
    request["queries"] = json!([record_query(
        "sizing",
        json!({"equipment":{"id":"sizing-source"}})
    )]);
    let (_, r) = f.execute(&request);
    let mut data = r["queries"][0]["value"]["data"].clone();
    data["case"]["inputs"]["gas_volume_flow"] = json!({"kind":"direct","value":{"quantity":"actual_volume_flow","value_si":0.9},"source":{"id":"source-test"}});
    request["steps"] = json!([sizing_step(),{"id":"edit","action":{"kind":"update_sizing_task","task":{"id":"sizing-source"},"expected_revision":1,"data":data}},adopt_step()]);
    request["steps"][2]["action"]["expected_revision"] = reference("edit", "task");
    request["queries"] = json!([results_query(
        "sizing",
        json!({"id":"sizing-source"}),
        json!({"id":"sizing-source-case"})
    )]);
    let (code, r) = f.execute(&request);
    assert_eq!(code, 3, "{r}");
    assert_eq!(r["error"]["details"]["reason"], "stale_calculation");
    assert_eq!(r["queries"][0]["value"]["status"], "stale");
    assert!(r["queries"][0]["value"].get("value").is_none());
}
#[test]
fn failed_calculation_retains_terminal_evidence_and_stops_later_actions() {
    let f = Fixture::new();
    let mut request=f.execution_request(&empty_project(),json!([
        {"id":"create","action":{"kind":"create_sizing_task"}},
        {"id":"size","action":{"kind":"size","task":reference("create","task"),"expected_revision":reference("create","task")}},
        {"id":"unreached","action":{"kind":"create_source"}}
    ]));
    request["queries"] = json!([results_query(
        "result",
        reference("create", "task"),
        reference("create", "case")
    )]);
    let (code, r) = f.execute(&request);
    assert_eq!(code, 4, "{r}");
    assert_eq!(r["steps"].as_array().unwrap().len(), 2);
    assert_eq!(r["steps"][1]["terminal"], "failed");
    assert_eq!(r["queries"][0]["value"]["status"], "failed");
    assert_eq!(
        r["queries"][0]["value"]["origin"]["execution"]["step"],
        "size"
    );
    assert!(
        !r["queries"][0]["value"]["issues"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
#[test]
fn flow_resources_are_explicit_exclusive_and_lazily_loaded() {
    let f = Fixture::new();
    let mut request = f.execution_request(
        &empty_project(),
        json!([{ "id":"source","action":{"kind":"create_source"}}]),
    );
    request["flow_resources"] = json!({"cache_root":"missing-cache","auth_cache_index":"missing-index","package":{"kind":"preferred"}});
    let (code, r) = f.execute(&request);
    assert_eq!(code, 2);
    assert_eq!(r["steps"], json!([]));
    request["steps"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"run","action":{"kind":"run_flow","expected_revision":0}}));
    let (code, r) = f.execute(&request);
    assert_eq!(code, 4, "{r}");
    assert_eq!(r["steps"][0]["outcome"], "committed");
    assert_eq!(r["steps"][1]["outcome"], "start_failed");
    request.as_object_mut().unwrap().remove("flow_resources");
    let (code, r) = f.execute(&request);
    assert_eq!(code, 2);
    assert_eq!(r["steps"], json!([]));
}

#[test]
fn independent_rating_rows_and_shared_geometry_have_distinct_invalidation_scopes() {
    let f = Fixture::new();
    let mut request = f.execution_request(&adoption_project(), json!([]));
    request["queries"] = json!([
        record_query(
            "case",
            json!({"case":{"task":{"id":"matrix-v2"},"case":{"id":"matrix-v2-case1"}}})
        ),
        record_query("geometry", json!({"geometry":{"id":"matrix-v2"}}))
    ]);
    let (_, initial) = f.execute(&request);
    let mut case = initial["queries"][0]["value"]["data"].clone();
    case["inputs"]["gas_volume_flow"] = json!({"kind":"direct","value":{"quantity":"actual_volume_flow","value_si":5.0},"source":{"id":"source-test"}});
    let mut geometry = initial["queries"][1]["value"]["data"].clone();
    geometry["inputs"]["internal_diameter"] = json!({"kind":"direct","value":{"quantity":"length","value_si":3.0},"source":{"id":"drawing"}});
    let rate = |id: &str, case: &str| json!({"id":id,"action":{"kind":"rate","task":{"id":"matrix-v2"},"case":{"id":case},"expected_case_revision":9,"expected_geometry_revision":6}});
    request["steps"] = json!([rate("one","matrix-v2-case1"),rate("two","matrix-v2-case2"),{"id":"edit","action":{"kind":"update_rating_case","task":{"id":"matrix-v2"},"case":{"id":"matrix-v2-case1"},"expected_revision":9,"data":case}}]);
    request["queries"] = json!([
        results_query(
            "one",
            json!({"id":"matrix-v2"}),
            json!({"id":"matrix-v2-case1"})
        ),
        results_query(
            "two",
            json!({"id":"matrix-v2"}),
            json!({"id":"matrix-v2-case2"})
        )
    ]);
    let (code, r) = f.execute(&request);
    assert_eq!(code, 0, "{r}");
    assert_eq!(r["queries"][0]["value"]["status"], "stale");
    assert_eq!(r["queries"][1]["value"]["status"], "current");
    assert!(r["queries"][0]["value"].get("value").is_none());
    request["steps"].as_array_mut().unwrap().push(json!({"id":"rerate","action":{"kind":"rate","task":{"id":"matrix-v2"},"case":{"id":"matrix-v2-case1"},"expected_case_revision":reference("edit","case"),"expected_geometry_revision":6}}));
    let (code, r) = f.execute(&request);
    assert_eq!(code, 0, "{r}");
    assert_eq!(
        r["queries"][0]["value"]["value"]["performance"]["conclusion"],
        "exceeded"
    );
    request["steps"].as_array_mut().unwrap().push(json!({"id":"geometry","action":{"kind":"update_rating_geometry","task":{"id":"matrix-v2"},"expected_revision":6,"data":geometry}}));
    let (code, r) = f.execute(&request);
    assert_eq!(code, 6, "{r}");
    request["steps"][4]["confirmation"] =
        json!({"digest":r["error"]["details"]["confirmation_digest"]});
    let (code, r) = f.execute(&request);
    assert_eq!(code, 0, "{r}");
    for q in r["queries"].as_array().unwrap() {
        assert_eq!(q["value"]["status"], "stale");
        assert!(q["value"].get("value").is_none());
    }
}

#[test]
fn adoption_digest_is_independent_of_allocated_task_case_and_copied_source_ids() {
    use super::super::super::{
        confirmation::Aliases, equipment, inputs, records::SizingData, references::Bindings,
    };
    use crate::run_actions::{EquipmentCalculationTarget, StudioRunAccess, StudioRunReply};
    use rf_ui::engineering::{EngineeringCommand, GeometryAdjustments, editor::*};
    fn review(churn: bool) -> Value {
        let StudioProjectOpen::Editable(mut app) =
            parse_studio_project(&adoption_project().to_string(), Path::new("fixture")).unwrap()
        else {
            panic!()
        };
        if churn {
            let created = equipment::apply(&mut app, EquipmentAction::CreateSource)
                .unwrap()
                .target
                .unwrap();
            let pending = equipment::apply(&mut app, EquipmentAction::Delete(created)).unwrap();
            let EquipmentActionOutcome::ConfirmationRequired { key, .. } = pending.outcome else {
                panic!()
            };
            equipment::apply(&mut app, EquipmentAction::Confirm(key)).unwrap();
            let mut source = app.workspace.engineering.inputs().basis.sources[0].clone();
            source.id = "adoption-source-4".into();
            source.revision = 0;
            app.workspace
                .engineering
                .apply(vec![EngineeringCommand::CreateSource(Box::new(source))])
                .unwrap();
            app.workspace
                .engineering
                .apply(vec![EngineeringCommand::DeleteSource(
                    "adoption-source-4".into(),
                )])
                .unwrap();
        }
        let before = app.clone();
        let created = equipment::apply(&mut app, EquipmentAction::CreateSizingTask).unwrap();
        let EquipmentEditorTarget::Task(id) = created.target.clone().unwrap() else {
            panic!()
        };
        let mut aliases = Aliases::default();
        aliases.created(
            "sizing",
            &before,
            &app,
            &equipment::outputs(&app, created.target.as_ref()),
        );
        let i = app.workspace.engineering.inputs();
        let mut data = SizingData::from(i.task(&"sizing-source".into()).unwrap());
        data.tag = "V-DYNAMIC".into();
        let updated = inputs::sizing(i.task(&id).unwrap(), &data, &Bindings::default()).unwrap();
        equipment::apply(
            &mut app,
            EquipmentAction::UpdateSizingTask(Box::new(updated)),
        )
        .unwrap();
        let expected = EquipmentActionContext::capture(&app.workspace.engineering);
        assert!(matches!(
            crate::StudioAppFacade::new().execute_equipment_run_action(
                &mut app,
                StudioRunAccess::Allowed,
                expected,
                EquipmentCalculationTarget::Sizing(id.clone())
            ),
            StudioRunReply::Completed(_)
        ));
        let candidate =
            aliases.candidate(app.workspace.engineering.current_candidate(&id).unwrap());
        let before = aliases.snapshot(&app);
        let old = app.clone();
        let prepared = equipment::apply(
            &mut app,
            EquipmentAction::PreviewAdoption {
                source: id,
                target: GeometryAdoptionDestination::New {
                    tag: "V-NEW".into(),
                    name: "source-ui-1 is a literal label".into(),
                    copy_case: true,
                },
                adjustments: GeometryAdjustments::default(),
            },
        )
        .unwrap();
        let proposal = app
            .workspace
            .equipment_editor
            .geometry_adoption_preview()
            .unwrap()
            .proposal()
            .clone();
        let EquipmentActionOutcome::AdoptionConfirmationRequired { key } = prepared.outcome else {
            panic!()
        };
        let receipt = equipment::apply(&mut app, EquipmentAction::ConfirmAdoption(key)).unwrap();
        aliases.created(
            "adopt",
            &old,
            &app,
            &equipment::outputs(&app, receipt.target.as_ref()),
        );
        aliases.adoption("adopt", &proposal, &app);
        json!({"before":before,"after":aliases.snapshot(&app),"candidate":candidate,"impacts":aliases.impacts(&receipt.impacts)})
    }
    assert_eq!(review(false), review(true));
}

#[test]
fn explicit_flow_run_returns_real_package_origin_and_edit_revokes_values() {
    let f = Fixture::new();
    let project: Value = serde_json::from_str(include_str!(
        "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
    ))
    .unwrap();
    let mut request = f.execution_request(&project, json!([]));
    let mut index = rf_store::StoredAuthCacheIndex::new(
        "https://example.invalid",
        "v3-test",
        rf_store::StoredCredentialReference::new("test", "v3-test"),
    );
    crate::test_support::write_official_binary_hydrocarbon_cached_package(
        &f.0.join("cache"),
        &mut index,
        crate::test_support::OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
        std::time::SystemTime::now(),
        None,
    );
    rf_store::write_auth_cache_index(f.0.join("index.json"), &index).unwrap();
    request["flow_resources"] = json!({"cache_root":"cache","auth_cache_index":"index.json","package":{"kind":"preferred"}});
    let revision = project["document"]["revision"].as_u64().unwrap();
    request["steps"] =
        json!([{ "id":"solve","action":{"kind":"run_flow","expected_revision":revision}}]);
    request["queries"] = json!([{ "id":"result","query":{"kind":"results","scope":"flow"}}]);
    let (code, r) = f.execute(&request);
    assert_eq!(code, 0, "{r}");
    assert_eq!(r["steps"][0]["terminal"], "succeeded");
    assert_eq!(r["queries"][0]["value"]["status"], "current");
    assert_eq!(
        r["queries"][0]["value"]["origin"]["execution"]["step"],
        "solve"
    );
    assert_eq!(
        r["queries"][0]["value"]["origin"]["execution"]["package"]["origin"],
        "cache"
    );
    let fields = r["queries"][0]["value"]["fields"].as_array().unwrap();
    assert!(!fields.is_empty());
    let unit = project["document"]["flowsheet"]["units"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap();
    request["steps"].as_array_mut().unwrap().push(json!({"id":"rename","action":{"kind":"write","expected_revision":revision,"target":{"object":{"unit":{"id":unit}},"section":"inputs","field":"name"},"value":"CLI changed name"}}));
    let (code, r) = f.execute(&request);
    assert_eq!(code, 0, "{r}");
    assert_eq!(r["queries"][0]["value"]["status"], "stale");
    assert!(r["queries"][0]["value"].get("fields").is_none());
}
