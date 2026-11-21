use rf_model::{engineering::*, equipment::*};
use rf_store::{FileOverwritePolicy, project_v3 as v3, project_v4 as v4};
use rf_types::SourceId;
use rf_unitops::heat_exchanger::concentric_tube::{ConcentricTubeRating, LimitCheck, rate_task};
use std::{fs, path::PathBuf};

const EQUIPMENT: v3::EquipmentSupport = v3::EquipmentSupport::ConcentricTubeRating;
const FLOW: v4::FlowPlanSupport = v4::FlowPlanSupport::Mixed;
const BASELINE_V3: &str =
    include_str!("../../../examples/flowsheets/standalone-separator-basis.rfproj.json");
const BASELINE_V4: &str =
    include_str!("../../../examples/flowsheets/fixed-tp-recycle-binary-hydrocarbon.rfproj.json");

struct TempDir(PathBuf);
impl TempDir {
    fn new() -> Self {
        let root = std::env::temp_dir();
        for index in 0.. {
            let path = root.join(format!(
                "rf-heat-exchanger-storage-{}-{index}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => panic!("create {}: {e}", path.display()),
            }
        }
        unreachable!()
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn benchmark_inputs(cold_molar_flow_mol_s: f64) -> v3::StoredEngineeringInputs {
    let source = SourceRecord {
        id: SourceId::new("nist-water-table-8"),
        revision: 3,
        kind: SourceKind::Literature,
        title: "NIST water properties, table 8".into(),
        reference: "https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=832373".into(),
        locator: "Table 8".into(),
        publication_version: String::new(),
        accessed_on: None,
        adoption_note: "Water at 298.15 K and 0.1 MPa".into(),
        review: SourceReview::Reviewed,
        applicability: SourceApplicability {
            fluid_system: "Water".into(),
            phase: SourcePhase::Liquid,
            operating_conditions: Some(SourceOperatingConditions {
                temperature_k: 298.15,
                pressure_pa: 100_000.0,
            }),
            scope: "Frozen liquid properties".into(),
        },
        observations: Vec::new(),
    };
    let mut geometry_source = source.clone();
    geometry_source.id = SourceId::new("van-zyl-test-section-3");
    geometry_source.title = "Van Zyl et al., test section 3".into();
    geometry_source.reference = "Van Zyl et al.".into();
    geometry_source.locator = "Table 5".into();
    geometry_source.adoption_note = "Concentric-tube benchmark dimensions".into();
    geometry_source.applicability.operating_conditions = None;
    geometry_source.applicability.scope = "Test section geometry".into();

    let side = |temperature_k, flow| ConcentricTubeSideInputs {
        inlet: ConcentricTubeLiquidInlet {
            temperature_k: Some(temperature_k),
            pressure_pa: Some(100_000.0),
            molar_flow_mol_s: Some(flow),
        },
        properties: ConcentricTubeLiquidProperties {
            source_id: source.id.clone(),
            molar_mass_kg_per_mol: Some(0.018_015_28),
            molar_heat_capacity_j_per_mol_k: Some(75.329_923_737_630_4),
            density_kg_per_m3: Some(997.047_013),
            dynamic_viscosity_pa_s: Some(0.000_889_996_774),
            thermal_conductivity_w_per_m_k: Some(0.607_198_237),
            liquid_domain: ConcentricTubeLiquidDomain {
                min_temperature_k: Some(293.15),
                max_temperature_k: Some(303.15),
                min_pressure_pa: Some(50_000.0),
                max_pressure_pa: Some(120_000.0),
            },
        },
    };
    let task = ConcentricTubeRatingTask {
        id: "h3-rating".into(),
        revision: 5,
        tag: "E-301".into(),
        name: "Concentric tube benchmark".into(),
        method: EquipmentMethod {
            id: CONCENTRIC_TUBE_RATING_METHOD_ID.into(),
            version: CONCENTRIC_TUBE_RATING_METHOD_VERSION,
        },
        geometry: ConcentricTubeGivenGeometry {
            revision: 7,
            source_id: Some(geometry_source.id.clone()),
            active_length_m: Some(4.85),
            inner_tube_inner_diameter_m: Some(0.014_46),
            inner_tube_outer_diameter_m: Some(0.015_88),
            outer_tube_inner_diameter_m: Some(0.026_76),
            wall_thermal_conductivity_w_per_m_k: Some(385.0),
        },
        case: ConcentricTubeRatingCase {
            flow_association: None,
            id: "h3-water".into(),
            revision: 11,
            name: "Clean water".into(),
            purpose: "Compare heat duty and straight-run pressure drops".into(),
            hot: side(303.15, 14.0),
            cold: side(293.15, cold_molar_flow_mol_s),
            limits: ConcentricTubeRatingLimits {
                min_heat_duty_w: Some(5_500.0),
                max_hot_pressure_drop_pa: Some(11_000.0),
                max_cold_pressure_drop_pa: Some(25_000.0),
            },
        },
    };
    v3::StoredEngineeringInputs {
        basis: EngineeringBasis {
            basis_entries: Vec::new(),
            sources: vec![source, geometry_source],
        },
        tasks: vec![task.into()],
    }
}

fn project_v3(inputs: Option<v3::StoredEngineeringInputs>) -> v3::ProjectV3Read {
    let v3::ProjectV3Read::Editable(mut project) = v3::parse(BASELINE_V3).unwrap() else {
        panic!("editable v3 baseline required");
    };
    project.engineering = inputs;
    v3::ProjectV3Read::Editable(project)
}

fn project_v4(inputs: Option<v3::StoredEngineeringInputs>) -> v4::ProjectV4Read {
    let v4::ProjectV4Read::Editable(mut project) = v4::parse(BASELINE_V4).unwrap() else {
        panic!("editable v4 baseline required");
    };
    project.engineering = inputs;
    v4::ProjectV4Read::Editable(project)
}

fn rate(inputs: &v3::StoredEngineeringInputs) -> ConcentricTubeRating {
    rate_task(
        inputs.tasks[0].as_concentric_tube_rating().unwrap(),
        &inputs.basis,
    )
    .unwrap()
}

fn check_benchmark(result: ConcentricTubeRating, cold_flow: f64) {
    let (ua, duty, hot_outlet_k, cold_outlet_k, cold_dp, cold_check) = if cold_flow == 40.0 {
        (
            874.553_423_114_86,
            5_521.707_636_437_1,
            297.914_262_742_51,
            294.982_508_040_12,
            23_679.130_040_479,
            LimitCheck::Passed,
        )
    } else {
        (
            945.093_778_801_02,
            5_877.489_522_370_7,
            297.576_906_873_93,
            294.710_466_075_30,
            34_962.547_685_264,
            LimitCheck::Failed,
        )
    };
    for (actual, expected, tolerance) in [
        (result.ua_w_per_k, ua, 1e-6),
        (result.heat_duty_w, duty, 1e-6),
        (result.hot.outlet.temperature_k, hot_outlet_k, 1e-8),
        (result.cold.outlet.temperature_k, cold_outlet_k, 1e-8),
        (
            result.hot.passage.pressure_drop_pa,
            9_799.896_961_973_8,
            1e-4,
        ),
        (result.cold.passage.pressure_drop_pa, cold_dp, 1e-4),
    ] {
        assert!(
            (actual - expected).abs() <= tolerance,
            "{actual} vs {expected}"
        );
    }
    assert_eq!(result.checks.heat_duty, LimitCheck::Passed);
    assert_eq!(result.checks.hot_pressure_drop, LimitCheck::Passed);
    assert_eq!(result.checks.cold_pressure_drop, cold_check);
}

#[test]
fn heat_exchanger_v3_h3_reopen_and_rate() {
    let temp = TempDir::new();
    let target = temp.0.join("v3.rfproj.json");
    for cold_flow in [40.0, 50.0] {
        let inputs = benchmark_inputs(cold_flow);
        let project = project_v3(Some(inputs.clone()));
        v3::write_with_equipment_support(
            &target,
            &project,
            FileOverwritePolicy::Allow,
            EQUIPMENT,
            false,
        )
        .unwrap();
        let reopened = v3::read_with_equipment_support(&target, EQUIPMENT).unwrap();
        assert_eq!(reopened, project);
        let v3::ProjectV3Read::Editable(reopened) = reopened else {
            panic!("expected editable v3")
        };
        let reopened_inputs = reopened.engineering.unwrap();
        let result = rate(&reopened_inputs);
        assert_eq!(result, rate(&inputs));
        check_benchmark(result, cold_flow);
    }
}

#[test]
fn heat_exchanger_v4_h3_reopen_and_rate() {
    let temp = TempDir::new();
    let target = temp.0.join("v4.rfproj.json");
    for cold_flow in [40.0, 50.0] {
        let inputs = benchmark_inputs(cold_flow);
        let project = project_v4(Some(inputs.clone()));
        let prepared = v4::prepare_write_with_equipment_support(
            &target,
            &project,
            FileOverwritePolicy::Allow,
            FLOW,
            EQUIPMENT,
        )
        .unwrap();
        assert!(!prepared.requires_upgrade());
        assert!(prepared.captures_project(&project).unwrap());
        v4::write_prepared(&prepared, false).unwrap();
        let reopened = v4::read_with_equipment_support(&target, FLOW, EQUIPMENT).unwrap();
        assert_eq!(reopened, project);
        let v4::ProjectV4Read::Editable(reopened) = reopened else {
            panic!("expected editable v4")
        };
        let reopened_inputs = reopened.engineering.unwrap();
        let result = rate(&reopened_inputs);
        assert_eq!(result, rate(&inputs));
        check_benchmark(result, cold_flow);
    }
}

#[test]
fn heat_exchanger_h3_incomplete_and_out_of_domain_records_stay_formal() {
    let mut inputs = benchmark_inputs(40.0);
    let EquipmentTask::ConcentricTubeRating(task) = &mut inputs.tasks[0] else {
        unreachable!()
    };
    task.geometry.active_length_m = None;
    task.case.cold.properties.liquid_domain.min_pressure_pa = Some(101_000.0);
    task.case.limits = ConcentricTubeRatingLimits::default();
    let project = project_v3(Some(inputs));
    let text = v3::to_pretty_json_with_equipment_support(&project, EQUIPMENT).unwrap();
    let v3::ProjectV3Read::Editable(reopened) =
        v3::parse_with_equipment_support(&text, EQUIPMENT).unwrap()
    else {
        panic!("expected editable v3")
    };
    let mut inputs = reopened.engineering.unwrap();
    let EquipmentTask::ConcentricTubeRating(task) = &mut inputs.tasks[0] else {
        unreachable!()
    };
    assert_eq!(task.geometry.active_length_m, None);
    assert_eq!(
        task.case.cold.properties.liquid_domain.min_pressure_pa,
        Some(101_000.0)
    );
    let missing = rate_task(task, &inputs.basis).unwrap_err();
    assert!(missing.message().contains("geometry.activeLengthM"));
    task.geometry.active_length_m = Some(4.85);
    assert!(rate_task(task, &inputs.basis).is_err());
    task.case.cold.properties.liquid_domain.min_pressure_pa = Some(50_000.0);
    let result = rate_task(task, &inputs.basis).unwrap();
    assert_eq!(result.checks.heat_duty, LimitCheck::NotSpecified);
    assert_eq!(result.checks.hot_pressure_drop, LimitCheck::NotSpecified);
    assert_eq!(result.checks.cold_pressure_drop, LimitCheck::NotSpecified);
}
