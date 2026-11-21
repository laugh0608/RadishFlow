use super::*;
use rf_model::{
    Component, Flowsheet, MaterialStreamState, UnitNode, UnitPort, engineering::SourceReview,
    equipment::ConcentricTubeLiquidDomain,
};
use rf_property_package::{
    InMemoryPropertyPackageProvider, PropertyPackageManifest, PropertyPackageSource,
};
use rf_store::{StoredDocumentMetadata, StoredProjectDocument};
use rf_thermo::{AntoineCoefficients, ThermoComponent, ThermoSystem};
use rf_types::{ComponentId, PhaseEquilibriumRegion, PhaseLabel, PortDirection, PortKind};
use rf_ui::{
    DocumentCommand, PhaseStateSnapshot, engineering::concentric_tube::LimitCheck,
    result_query::ResultStatus,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const PACKAGE: &str = "h3-flow-water-test";
const WATER: &str = "water";
const HOT: &str = "hot-feed";
const COLD: &str = "cold-feed";

struct Fixture {
    path: PathBuf,
    provider: InMemoryPropertyPackageProvider,
    qualification: H3PureWaterQualification,
}

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "rf-h3-flow-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();

        let provider = water_provider(75.329_923_737_630_4);
        let package = provider.load_package(PACKAGE, SystemTime::now()).unwrap();
        let source = H3WaterPropertySource {
            id: "nist-water-table-8".into(),
            revision: 3,
        };
        let qualification = H3PureWaterQualification {
            component_id: WATER.into(),
            package_id: package.manifest().package_id.clone(),
            package_version: package.manifest().version.clone(),
            payload_fingerprint: package.provenance().payload_fingerprint.clone(),
            phase_reference: "https://webbook.nist.gov/cgi/cbook.cgi?ID=C7732185&Type=ANTOINE"
                .into(),
            phase_domain: ConcentricTubeLiquidDomain {
                min_temperature_k: Some(293.15),
                max_temperature_k: Some(303.15),
                min_pressure_pa: Some(50_000.0),
                max_pressure_pa: Some(120_000.0),
            },
            hot_properties: source.clone(),
            cold_properties: source,
        };
        let fixture = Self {
            path: directory.join("water.rfproj.json"),
            provider,
            qualification,
        };
        fixture.write_project();
        fixture
    }

    fn write_project(&self) {
        let mut flowsheet = Flowsheet::new("H3 pure water");
        flowsheet
            .set_property_package_id(Some(PACKAGE.into()))
            .unwrap();
        flowsheet
            .insert_component(Component::new(WATER, "Water").with_formula("H2O"))
            .unwrap();
        for (id, label, temperature, flow) in [
            (HOT, "Hot feed", 303.15, 14.0),
            (COLD, "Cold feed", 293.15, 40.0),
        ] {
            flowsheet
                .insert_stream(MaterialStreamState::from_tpzf(
                    id,
                    label,
                    temperature,
                    100_000.0,
                    flow,
                    [(ComponentId::new(WATER), 1.0)].into(),
                ))
                .unwrap();
            flowsheet
                .insert_unit(UnitNode::new(
                    format!("{id}-unit"),
                    label,
                    "feed",
                    vec![UnitPort::new(
                        "outlet",
                        PortDirection::Outlet,
                        PortKind::Material,
                        Some(id.into()),
                    )],
                ))
                .unwrap();
        }
        let app = AppState::new(rf_ui::FlowsheetDocument::new(
            flowsheet.clone(),
            rf_ui::DocumentMetadata::new("h3-water-v4", "H3 pure water", UNIX_EPOCH),
        ));
        let project = project_v4::StoredProjectV4 {
            document: StoredProjectDocument {
                revision: 0,
                flowsheet,
                metadata: StoredDocumentMetadata::new("h3-water-v4", "H3 pure water", UNIX_EPOCH),
            },
            display_units: Default::default(),
            engineering: None,
            steady_state_plan: StoredFlowPlan::Acyclic,
            object_identities: Some(app.workspace.prepare_object_identities().unwrap()),
            variable_watch_list: None,
        };
        let json = project_v4::to_pretty_json(&ProjectV4Read::Editable(Box::new(project))).unwrap();
        let mut root: Value = serde_json::from_str(&json).unwrap();
        root["requiredCapabilities"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":project_v4::CONCENTRIC_TUBE_RATING_CAPABILITY,"version":1}));
        root["engineering"] = serde_json::from_str(include_str!(
            "../../../tests/fixtures/h3-review-engineering.json"
        ))
        .unwrap();
        root["engineering"]["tasks"]
            .as_array_mut()
            .unwrap()
            .truncate(1);
        fs::write(&self.path, serde_json::to_string_pretty(&root).unwrap()).unwrap();
    }

    fn open(&self) -> H3FlowReview {
        H3FlowReview::open(&self.path, self.qualification.clone()).unwrap()
    }

    fn solved(&self) -> H3FlowReview {
        let mut review = self.open();
        review.solve_flow(&self.provider).unwrap();
        assert_eq!(
            review.workspace().flow_result().status(),
            ResultStatus::Current
        );
        review
    }

    fn amend_project(&self, amend: impl FnOnce(&mut Value)) {
        let mut root: Value = serde_json::from_slice(&fs::read(&self.path).unwrap()).unwrap();
        amend(&mut root);
        fs::write(&self.path, serde_json::to_string_pretty(&root).unwrap()).unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(self.path.parent().unwrap());
    }
}

fn water_provider(heat_capacity: f64) -> InMemoryPropertyPackageProvider {
    let mut water = ThermoComponent::new(WATER, "Water");
    // Gubkov et al., as tabulated by NIST for 293–343 K. Convert
    // log10(bar) to the existing ln(kPa) correlation.
    let ln_10 = std::f64::consts::LN_10;
    water.antoine = Some(AntoineCoefficients::new(
        (6.20963 + 2.0) * ln_10,
        2354.731 * ln_10,
        7.559,
    ));
    water.liquid_heat_capacity_j_per_mol_k = Some(heat_capacity);
    water.vapor_heat_capacity_j_per_mol_k = Some(33.6);
    let manifest = PropertyPackageManifest::new(
        PACKAGE,
        "1",
        PropertyPackageSource::LocalBundled,
        vec![WATER.into()],
    );
    InMemoryPropertyPackageProvider::try_new(vec![(manifest, ThermoSystem::new(vec![water]))])
        .unwrap()
}

fn member() -> EquipmentCaseKey {
    EquipmentCaseKey {
        task_id: "h3-rating".into(),
        case_id: "h3-water".into(),
    }
}

fn stream(review: &H3FlowReview, id: &str) -> H3FlowStream {
    let id = StreamId::new(id);
    let creation_id = review
        .workspace()
        .object_creation_id(&ObjectId::Stream(id.clone()))
        .unwrap()
        .clone();
    H3FlowStream { id, creation_id }
}

fn preview(review: &H3FlowReview) -> RfResult<H3FlowPreview> {
    review.preview(member(), stream(review, HOT), stream(review, COLD))
}

fn assert_code(error: RfError, code: &str) {
    assert_eq!(
        error.context().diagnostic_code(),
        Some(format!("equipment.h3.flow.{code}").as_str()),
        "{error}"
    );
}

fn assert_cold_code(error: RfError, code: &str) {
    assert_eq!(error.context().related_stream_ids(), &[StreamId::new(COLD)]);
    assert_code(error, code);
}

fn edit_flow(review: &mut H3FlowReview, flow: f64) {
    let mut next = review.workspace().document.flowsheet.clone();
    next.streams
        .get_mut(&COLD.into())
        .unwrap()
        .total_molar_flow_mol_s = flow;
    review
        .flow
        .commit_document_change(
            DocumentCommand::SetFeedMolarFlows {
                values: vec![(COLD.into(), flow)],
            },
            next,
            UNIX_EPOCH,
        )
        .unwrap();
}

#[test]
fn h3_flow_rates_the_two_accepted_loads_and_preserves_the_v4_bytes() {
    let fixture = Fixture::new();
    let bytes = fs::read(&fixture.path).unwrap();
    let mut review = fixture.open();
    assert_code(preview(&review).unwrap_err(), "flow_unavailable");
    review.solve_flow(&fixture.provider).unwrap();
    let first_preview = preview(&review).unwrap();
    let old_preview = preview(&review).unwrap();
    assert_eq!(first_preview.hot().1.molar_flow_mol_s, Some(14.0));
    assert_eq!(first_preview.hot().1.temperature_k, Some(303.15));
    assert_eq!(first_preview.cold().1.molar_flow_mol_s, Some(40.0));
    assert_eq!(first_preview.cold().1.temperature_k, Some(293.15));
    assert_eq!(first_preview.cold().1.pressure_pa, Some(100_000.0));
    assert_eq!(first_preview.project_sha256(), rf_store::sha256_hex(&bytes));
    assert_eq!(first_preview.origin().document_id().as_str(), "h3-water-v4");
    let first_run = first_preview.origin().execution().unwrap().run_id.clone();
    let first_result = review.rate(first_preview).unwrap();
    assert_benchmark(review.current_rating(&first_result).unwrap(), 40);

    edit_flow(&mut review, 50.0);
    assert_code(preview(&review).unwrap_err(), "flow_unavailable");
    assert_code(
        review.current_rating(&first_result).unwrap_err(),
        "inputs_changed",
    );
    assert_code(review.rate(old_preview).unwrap_err(), "inputs_changed");
    review.solve_flow(&fixture.provider).unwrap();
    let second_preview = preview(&review).unwrap();
    assert_eq!(second_preview.cold().1.molar_flow_mol_s, Some(50.0));
    assert_ne!(
        second_preview.origin().execution().unwrap().run_id,
        first_run
    );
    assert_code(
        review.current_rating(&first_result).unwrap_err(),
        "inputs_changed",
    );
    let second_result = review.rate(second_preview).unwrap();
    assert_benchmark(review.current_rating(&second_result).unwrap(), 50);
    assert_eq!(
        review
            .cases()
            .next()
            .unwrap()
            .case
            .cold
            .inlet
            .molar_flow_mol_s,
        Some(40.0)
    );
    assert_eq!(
        review.workspace().document.flowsheet.streams[&COLD.into()].total_molar_flow_mol_s,
        50.0
    );
    assert_eq!(fs::read(&fixture.path).unwrap(), bytes);
}

fn assert_benchmark(rating: &ConcentricTubeRating, cold_flow: i32) {
    let (duty, ua, hot, cold, dp, limit) = if cold_flow == 40 {
        (
            5_521.707_636_437_1,
            874.553_423_114_86,
            297.914_262_742_51,
            294.982_508_040_12,
            23_679.130_040_479,
            LimitCheck::Passed,
        )
    } else {
        (
            5_877.489_522_370_7,
            945.093_778_801_02,
            297.576_906_873_93,
            294.710_466_075_30,
            34_962.547_685_264,
            LimitCheck::Failed,
        )
    };
    assert!((rating.heat_duty_w - duty).abs() <= 1e-6);
    assert!((rating.ua_w_per_k - ua).abs() <= 1e-6);
    assert!((rating.hot.outlet.temperature_k - hot).abs() <= 1e-8);
    assert!((rating.cold.outlet.temperature_k - cold).abs() <= 1e-8);
    assert!((rating.hot.passage.pressure_drop_pa - 9_799.896_961_973_8).abs() <= 1e-4);
    assert!((rating.cold.passage.pressure_drop_pa - dp).abs() <= 1e-4);
    assert_eq!(rating.checks.heat_duty, LimitCheck::Passed);
    assert_eq!(rating.checks.hot_pressure_drop, LimitCheck::Passed);
    assert_eq!(rating.checks.cold_pressure_drop, limit);
}

#[test]
fn h3_flow_rejects_undo_same_input_rerun_and_failed_rerun_evidence() {
    let fixture = Fixture::new();
    let original = fs::read(&fixture.path).unwrap();
    let mut review = fixture.solved();
    let before_edit = preview(&review).unwrap();
    let rating = review.rate(preview(&review).unwrap()).unwrap();
    edit_flow(&mut review, 50.0);
    review
        .flow
        .undo_document_command(UNIX_EPOCH)
        .unwrap()
        .unwrap();
    assert_code(preview(&review).unwrap_err(), "flow_unavailable");
    review.solve_flow(&fixture.provider).unwrap();
    assert_eq!(
        preview(&review).unwrap().cold().1.molar_flow_mol_s,
        Some(40.0)
    );
    assert_code(review.rate(before_edit).unwrap_err(), "inputs_changed");
    assert_code(
        review.current_rating(&rating).unwrap_err(),
        "inputs_changed",
    );

    let before_rerun = preview(&review).unwrap();
    let old_seq = before_rerun.sequence();
    review.solve_flow(&fixture.provider).unwrap();
    assert!(preview(&review).unwrap().sequence() > old_seq);
    assert_code(review.rate(before_rerun).unwrap_err(), "inputs_changed");
    let rating = review.rate(preview(&review).unwrap()).unwrap();
    assert!(
        review
            .solve_flow(&InMemoryPropertyPackageProvider::default())
            .is_err()
    );
    assert_eq!(
        review.workspace().flow_result().status(),
        ResultStatus::Failed
    );
    assert_code(preview(&review).unwrap_err(), "flow_unavailable");
    assert_code(
        review.current_rating(&rating).unwrap_err(),
        "inputs_changed",
    );
    assert_eq!(fs::read(&fixture.path).unwrap(), original);
}

#[test]
fn h3_flow_requires_distinct_streams_and_rejects_recreated_objects_and_reopen() {
    let fixture = Fixture::new();
    let bytes = fs::read(&fixture.path).unwrap();
    let mut review = fixture.solved();
    let original_cold = stream(&review, COLD);
    let hot = stream(&review, HOT);
    assert_code(
        review
            .preview(member(), hot.clone(), hot.clone())
            .unwrap_err(),
        "same_stream",
    );
    let missing_case = EquipmentCaseKey {
        case_id: "other".into(),
        ..member()
    };
    assert_code(
        review
            .preview(missing_case, hot.clone(), original_cold.clone())
            .unwrap_err(),
        "case_missing",
    );

    let old_preview = preview(&review).unwrap();
    let old_rating = review.rate(preview(&review).unwrap()).unwrap();
    let mut original = review.workspace().document.flowsheet.clone();
    let unit_id = format!("{COLD}-unit").into();
    let mut removed = original.clone();
    removed.remove_stream(&original_cold.id).unwrap();
    removed.units.get_mut(&unit_id).unwrap().ports[0].stream_id = None;
    review
        .flow
        .commit_document_change(
            DocumentCommand::DeleteStream {
                stream_id: original_cold.id.clone(),
            },
            removed,
            UNIX_EPOCH,
        )
        .unwrap();
    let recreated_id = StreamId::new("replacement-cold-feed");
    let mut recreated_stream = original.remove_stream(&original_cold.id).unwrap();
    let label = recreated_stream.name.clone();
    recreated_stream.id = recreated_id.clone();
    original.insert_stream(recreated_stream).unwrap();
    original.units.get_mut(&unit_id).unwrap().ports[0].stream_id = Some(recreated_id.clone());
    review
        .flow
        .commit_document_change(
            DocumentCommand::ConnectPorts {
                stream_id: recreated_id.clone(),
                from_unit_id: unit_id,
                from_port: "outlet".into(),
                to_unit_id: None,
                to_port: None,
            },
            original,
            UNIX_EPOCH,
        )
        .unwrap();
    review.solve_flow(&fixture.provider).unwrap();
    assert_eq!(
        review.workspace().document.flowsheet.streams[&recreated_id].name,
        label
    );
    let recreated = stream(&review, recreated_id.as_str());
    assert_ne!(original_cold.creation_id, recreated.creation_id);
    assert_cold_code(
        review.preview(member(), hot, original_cold).unwrap_err(),
        "stream_changed",
    );
    assert_code(
        review.current_rating(&old_rating).unwrap_err(),
        "inputs_changed",
    );
    let replacement_preview = review
        .preview(member(), stream(&review, HOT), recreated)
        .unwrap();
    let replacement_rating = review.rate(replacement_preview).unwrap();
    assert_benchmark(review.current_rating(&replacement_rating).unwrap(), 40);

    let mut reopened = fixture.open();
    assert_code(preview(&reopened).unwrap_err(), "flow_unavailable");
    assert_code(
        reopened.current_rating(&old_rating).unwrap_err(),
        "inputs_changed",
    );
    reopened.solve_flow(&fixture.provider).unwrap();
    let new_preview = preview(&reopened).unwrap();
    assert_eq!(new_preview.sequence(), old_preview.sequence());
    assert_eq!(new_preview.snapshot_id(), old_preview.snapshot_id());
    assert_eq!(new_preview.project_sha256(), old_preview.project_sha256());
    assert_code(reopened.rate(old_preview).unwrap_err(), "inputs_changed");
    assert_eq!(fs::read(&fixture.path).unwrap(), bytes);
}

#[test]
fn h3_flow_verifies_the_executed_package_and_both_property_sources() {
    let mut fixture = Fixture::new();
    let mut review = fixture.solved();
    let old_result = review.rate(preview(&review).unwrap()).unwrap();
    let replacement = water_provider(76.0);
    // Same package ID and version, with a different validated payload.
    review.solve_flow(&replacement).unwrap();
    assert_eq!(
        review.workspace().flow_result().status(),
        ResultStatus::Current
    );
    assert_code(preview(&review).unwrap_err(), "package_unqualified");
    assert_code(
        review.current_rating(&old_result).unwrap_err(),
        "inputs_changed",
    );

    fixture.qualification.cold_properties.revision += 1;
    assert_cold_code(
        preview(&fixture.solved()).unwrap_err(),
        "property_source_unqualified",
    );
    fixture.qualification.cold_properties.revision -= 1;
    fixture.amend_project(|root| {
        root["engineering"]["sources"][0]["review"] = json!(SourceReview::Unreviewed);
    });
    let error = preview(&fixture.solved()).unwrap_err();
    assert_eq!(error.context().related_stream_ids(), &[StreamId::new(HOT)]);
    assert_code(error, "property_source_unqualified");
}

#[test]
fn h3_flow_rejects_missing_or_incompatible_stream_phase_composition_and_domains() {
    let mut fixture = Fixture::new();
    let review = fixture.solved();
    let FlowResultView::Current { snapshot, .. } = review.workspace().flow_result() else {
        panic!("expected accepted source");
    };
    let record = snapshot
        .streams
        .iter()
        .find(|row| row.stream_id.as_str() == COLD)
        .unwrap();
    let properties = &review.saved.case(&member()).unwrap().case.cold.properties;
    let check = |stream: &rf_ui::StreamStateSnapshot| {
        review
            .qualification
            .inlet(stream, properties, review.saved.basis(), Side::Cold)
    };
    assert_eq!(check(record).unwrap().molar_flow_mol_s, Some(40.0));
    let mut candidate = record.clone();
    candidate.bubble_dew_window = None;
    assert_cold_code(check(&candidate).unwrap_err(), "phase_missing");
    for phase_region in [
        PhaseEquilibriumRegion::VaporOnly,
        PhaseEquilibriumRegion::TwoPhase,
    ] {
        let mut candidate = record.clone();
        candidate.bubble_dew_window.as_mut().unwrap().phase_region = phase_region;
        assert_cold_code(check(&candidate).unwrap_err(), "not_single_liquid");
    }
    let mut candidate = record.clone();
    candidate.phases.push(PhaseStateSnapshot {
        label: PhaseLabel::Vapor.as_str().into(),
        phase_fraction: 0.01,
        composition: candidate.overall_mole_fractions.clone(),
        molar_enthalpy_j_per_mol: None,
    });
    assert_cold_code(check(&candidate).unwrap_err(), "not_single_liquid");
    let mut candidate = record.clone();
    candidate.pressure_pa = candidate
        .bubble_dew_window
        .as_ref()
        .unwrap()
        .bubble_pressure_pa;
    assert_cold_code(check(&candidate).unwrap_err(), "not_single_liquid");
    let mut candidate = record.clone();
    candidate.overall_mole_fractions = vec![(WATER.into(), 0.99), ("other".into(), 0.01)];
    assert_cold_code(check(&candidate).unwrap_err(), "not_pure_water");
    candidate = record.clone();
    candidate.total_molar_flow_mol_s = 0.0;
    assert_cold_code(check(&candidate).unwrap_err(), "invalid_flow");

    fixture.qualification.phase_domain.min_temperature_k = Some(294.0);
    assert_cold_code(
        preview(&fixture.solved()).unwrap_err(),
        "outside_phase_domain",
    );
    fixture.qualification.phase_domain.min_temperature_k = Some(293.15);
    fixture.qualification.phase_domain.min_pressure_pa = Some(85_000.0);
    let narrowed = fixture.solved();
    assert_cold_code(
        narrowed.rate(preview(&narrowed).unwrap()).unwrap_err(),
        "outlet_outside_phase_domain",
    );
    fixture.qualification.phase_domain.min_pressure_pa = Some(50_000.0);
    fixture.amend_project(|root| {
        root["engineering"]["tasks"][0]["case"]["cold"]["properties"]["liquidDomain"]["maxPressurePa"] = json!(95_000);
    });
    assert_cold_code(
        preview(&fixture.solved()).unwrap_err(),
        "outside_property_domain",
    );
}

#[test]
fn h3_flow_rejects_unsupported_project_envelopes_and_incomplete_qualification() {
    let fixture = Fixture::new();
    let mut incomplete = fixture.qualification.clone();
    incomplete.phase_reference.clear();
    assert_code(
        H3FlowReview::open(&fixture.path, incomplete).unwrap_err(),
        "qualification_incomplete",
    );
    fixture.amend_project(|root| {
        root.as_object_mut().unwrap().remove("objectIdentities");
        root["requiredCapabilities"]
            .as_array_mut()
            .unwrap()
            .retain(|cap| cap["id"] != project_v4::OBJECT_IDENTITY_CAPABILITY);
    });
    assert_code(
        H3FlowReview::open(&fixture.path, fixture.qualification.clone()).unwrap_err(),
        "unsupported_project",
    );
    fixture.amend_project(|root| root["schemaVersion"] = json!(3));
    assert_code(
        H3FlowReview::open(&fixture.path, fixture.qualification.clone()).unwrap_err(),
        "unsupported_project",
    );
}
