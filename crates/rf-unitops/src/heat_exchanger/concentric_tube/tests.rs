use super::*;

// Table 5, test section 3, from Van Zyl et al. The operating conditions are synthetic.
const GEOMETRY: ConcentricTubeGeometry = ConcentricTubeGeometry {
    active_length_m: 4.85,
    inner_tube_inner_diameter_m: 0.01446,
    inner_tube_outer_diameter_m: 0.01588,
    outer_tube_inner_diameter_m: 0.02676,
    wall_thermal_conductivity_w_per_m_k: 385.0,
};

const WATER: ConstantLiquidProperties<'static> = ConstantLiquidProperties {
    molar_mass_kg_per_mol: 0.018_015_28,
    molar_heat_capacity_j_per_mol_k: 75.329_923_737_630_4,
    density_kg_per_m3: 997.047_013,
    dynamic_viscosity_pa_s: 0.000_889_996_774,
    thermal_conductivity_w_per_m_k: 0.607_198_237,
    domain: LiquidPropertyDomain {
        min_temperature_k: 293.15,
        max_temperature_k: 303.15,
        min_pressure_pa: 50_000.0,
        max_pressure_pa: 120_000.0,
    },
    source: "Patek et al. (2009), Table 8, 298.15 K and 0.1 MPa",
};

const LIMITS: RatingLimits = RatingLimits {
    min_heat_duty_w: Some(5500.0),
    max_hot_pressure_drop_pa: Some(11_000.0),
    max_cold_pressure_drop_pa: Some(25_000.0),
};

fn water_side(temperature_k: f64, molar_flow_mol_s: f64) -> LiquidSide<'static> {
    LiquidSide {
        inlet: LiquidState {
            temperature_k,
            pressure_pa: 100_000.0,
            molar_flow_mol_s,
        },
        properties: WATER,
    }
}

fn benchmark_sides() -> (LiquidSide<'static>, LiquidSide<'static>) {
    (water_side(303.15, 14.0), water_side(293.15, 40.0))
}

fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} differs from {expected} by more than {tolerance}"
    );
}

fn assert_error(result: RfResult<ConcentricTubeRating>, code: &str, location: &str) {
    let err = result.unwrap_err();
    let diagnostic_code = format!("unit.heat_exchanger.concentric_tube.{code}");
    assert_eq!(
        err.context().diagnostic_code(),
        Some(diagnostic_code.as_str())
    );
    assert!(err.to_string().contains(location), "{err}");
}

#[test]
fn rating_matches_independent_lmtd_and_darcy_benchmark() {
    let (hot, cold) = benchmark_sides();
    let rating = rate_countercurrent(GEOMETRY, hot, cold, LIMITS).unwrap();

    close(rating.heat_duty_w, 5_521.707_636_437_1, 1e-6);
    close(rating.ua_w_per_k, 874.553_423_114_86, 1e-6);
    close(
        rating.inner_heat_transfer_area_m2,
        0.220_323_034_388_91,
        1e-12,
    );
    close(
        rating.outer_heat_transfer_area_m2,
        0.241_959_182_994_18,
        1e-12,
    );
    close(
        rating.overall_heat_transfer_coefficient_outer_w_per_m2_k,
        3_614.466_755_477_1,
        1e-7,
    );
    close(rating.lmtd_k, 6.313_745_381_923_8, 1e-8);
    close(rating.hot.passage.reynolds_number, 24_952.981_544, 1e-7);
    close(
        rating.cold.passage.reynolds_number,
        24_177.171_879_271,
        1e-7,
    );
    close(rating.hot.passage.nusselt_number, 170.551_620_430_09, 1e-8);
    close(rating.cold.passage.nusselt_number, 147.608_637_465_64, 1e-8);
    close(
        rating.hot.passage.darcy_friction_factor,
        0.024_700_658_223_292,
        1e-10,
    );
    close(
        rating.cold.passage.darcy_friction_factor,
        0.027_081_234_739_333,
        1e-10,
    );

    for (inlet, side, duty_w, outlet_k, pressure_drop_pa) in [
        (
            hot.inlet,
            rating.hot,
            -rating.heat_duty_w,
            297.914_262_742_51,
            9_799.896_961_973_8,
        ),
        (
            cold.inlet,
            rating.cold,
            rating.heat_duty_w,
            294.982_508_040_12,
            23_679.130_040_479,
        ),
    ] {
        close(side.outlet.temperature_k, outlet_k, 1e-8);
        close(side.passage.pressure_drop_pa, pressure_drop_pa, 1e-4);
        close(
            side.outlet.pressure_pa,
            inlet.pressure_pa - pressure_drop_pa,
            1e-4,
        );
        assert_eq!(side.outlet.molar_flow_mol_s, inlet.molar_flow_mol_s);
        let actual_gain_w = inlet.molar_flow_mol_s
            * WATER.molar_heat_capacity_j_per_mol_k
            * (side.outlet.temperature_k - inlet.temperature_k);
        close(actual_gain_w, duty_w, 1e-6);
        close(
            side.sensible_energy_residual_w,
            actual_gain_w - duty_w,
            1e-10,
        );
    }
    close(rating.overall_sensible_energy_residual_w, 0.0, 1e-6);
    close(rating.lmtd_heat_duty_residual_w, 0.0, 1e-6);
    assert_eq!(
        rating.checks,
        LimitChecks {
            heat_duty: LimitCheck::Passed,
            hot_pressure_drop: LimitCheck::Passed,
            cold_pressure_drop: LimitCheck::Passed,
        }
    );
}

#[test]
fn increased_annular_flow_preserves_results_when_pressure_limit_fails() {
    let (hot, mut cold) = benchmark_sides();
    cold.inlet.molar_flow_mol_s = 50.0;
    let rating = rate_countercurrent(GEOMETRY, hot, cold, LIMITS).unwrap();
    close(rating.heat_duty_w, 5_877.489_522_370_7, 1e-6);
    close(rating.ua_w_per_k, 945.093_778_801_02, 1e-6);
    close(rating.hot.outlet.temperature_k, 297.576_906_873_93, 1e-8);
    close(rating.cold.outlet.temperature_k, 294.710_466_075_30, 1e-8);
    close(
        rating.hot.passage.pressure_drop_pa,
        9_799.896_961_973_8,
        1e-4,
    );
    close(
        rating.cold.passage.pressure_drop_pa,
        34_962.547_685_264,
        1e-4,
    );
    assert_eq!(
        rating.checks,
        LimitChecks {
            heat_duty: LimitCheck::Passed,
            hot_pressure_drop: LimitCheck::Passed,
            cold_pressure_drop: LimitCheck::Failed,
        }
    );
}

#[test]
fn annular_model_reproduces_the_calculations_compared_to_figures_9_and_13() {
    let (mut hot, mut cold) = benchmark_sides();
    let flow_factor = PI * WATER.dynamic_viscosity_pa_s / (4.0 * WATER.molar_mass_kg_per_mol);
    hot.inlet.molar_flow_mol_s = 20_000.0 * flow_factor * GEOMETRY.inner_tube_inner_diameter_m;
    let annular_diameter_sum =
        GEOMETRY.outer_tube_inner_diameter_m + GEOMETRY.inner_tube_outer_diameter_m;

    cold.inlet.molar_flow_mol_s = 25_000.0 * flow_factor * annular_diameter_sum;
    let figure_9 = rate_countercurrent(GEOMETRY, hot, cold, RatingLimits::default()).unwrap();
    close(figure_9.cold.passage.reynolds_number, 25_000.0, 1e-8);
    close(
        figure_9.cold.passage.nusselt_number,
        151.790_764_974_519_6,
        1e-8,
    );

    cold.inlet.molar_flow_mol_s = 20_000.0 * flow_factor * annular_diameter_sum;
    let figure_13 = rate_countercurrent(GEOMETRY, hot, cold, RatingLimits::default()).unwrap();
    close(figure_13.cold.passage.reynolds_number, 20_000.0, 1e-8);
    close(
        figure_13.cold.passage.darcy_friction_factor,
        0.028_452_840_410_500_875,
        1e-10,
    );
}

#[test]
fn equal_and_nearly_equal_heat_capacity_rates_are_continuous() {
    let hot = water_side(303.15, 20.0);
    for cold_flow in [20.0, 20.0_f64.next_up(), 20.0_f64.next_down()] {
        let cold = water_side(293.15, cold_flow);
        let rating = rate_countercurrent(GEOMETRY, hot, cold, RatingLimits::default()).unwrap();
        close(rating.ua_w_per_k, 731.811_123_623_20, 1e-6);
        close(rating.heat_duty_w, 4_925.575_388_247_6, 1e-6);
        close(
            rating.hot_inlet_approach_k,
            rating.cold_inlet_approach_k,
            1e-12,
        );
        close(rating.lmtd_heat_duty_residual_w, 0.0, 1e-6);
    }
}

#[test]
fn equal_inlet_temperatures_keep_positive_ua_and_hydraulic_losses() {
    let (mut hot, cold) = benchmark_sides();
    hot.inlet.temperature_k = cold.inlet.temperature_k;
    let rating = rate_countercurrent(GEOMETRY, hot, cold, RatingLimits::default()).unwrap();
    assert_eq!(rating.heat_duty_w, 0.0);
    assert_eq!(rating.lmtd_k, 0.0);
    assert_eq!(rating.hot.outlet.temperature_k, hot.inlet.temperature_k);
    assert_eq!(rating.cold.outlet.temperature_k, cold.inlet.temperature_k);
    close(rating.ua_w_per_k, 874.553_423_114_86, 1e-6);
    close(
        rating.hot.passage.pressure_drop_pa,
        9_799.896_961_973_8,
        1e-4,
    );
    close(
        rating.cold.passage.pressure_drop_pa,
        23_679.130_040_479,
        1e-4,
    );
}

#[test]
fn limits_distinguish_unset_equal_and_failed() {
    let (hot, cold) = benchmark_sides();
    let rating = rate_countercurrent(GEOMETRY, hot, cold, RatingLimits::default()).unwrap();
    assert_eq!(
        rating.checks,
        LimitChecks {
            heat_duty: LimitCheck::NotSpecified,
            hot_pressure_drop: LimitCheck::NotSpecified,
            cold_pressure_drop: LimitCheck::NotSpecified,
        }
    );
    let limits = RatingLimits {
        min_heat_duty_w: Some(rating.heat_duty_w.next_up()),
        max_hot_pressure_drop_pa: Some(rating.hot.passage.pressure_drop_pa.next_down()),
        max_cold_pressure_drop_pa: Some(rating.cold.passage.pressure_drop_pa),
    };
    let rating = rate_countercurrent(GEOMETRY, hot, cold, limits).unwrap();
    assert_eq!(
        rating.checks,
        LimitChecks {
            heat_duty: LimitCheck::Failed,
            hot_pressure_drop: LimitCheck::Failed,
            cold_pressure_drop: LimitCheck::Passed,
        }
    );
    assert_error(
        rate_countercurrent(
            GEOMETRY,
            hot,
            cold,
            RatingLimits {
                min_heat_duty_w: Some(f64::NAN),
                ..LIMITS
            },
        ),
        "invalid_limit",
        "minimum heat duty",
    );
}

#[test]
fn rejects_bad_geometry_and_geometric_correlation_bounds() {
    let (hot, cold) = benchmark_sides();
    for (geometry, code, location) in [
        (
            ConcentricTubeGeometry {
                inner_tube_inner_diameter_m: 0.01588,
                ..GEOMETRY
            },
            "invalid_geometry",
            "diameters",
        ),
        (
            ConcentricTubeGeometry {
                wall_thermal_conductivity_w_per_m_k: 0.0,
                ..GEOMETRY
            },
            "invalid_geometry",
            "conductivity",
        ),
        (
            ConcentricTubeGeometry {
                outer_tube_inner_diameter_m: 0.034,
                ..GEOMETRY
            },
            "outside_correlation_domain",
            "diameter ratio",
        ),
        (
            ConcentricTubeGeometry {
                active_length_m: 1.4,
                ..GEOMETRY
            },
            "outside_correlation_domain",
            "L/Dh",
        ),
    ] {
        assert_error(
            rate_countercurrent(geometry, hot, cold, LIMITS),
            code,
            location,
        );
    }
}

#[test]
fn rejects_invalid_case_inputs_and_identifies_the_side() {
    let (hot, cold) = benchmark_sides();
    let mut invalid = cold;
    invalid.properties.source = " ";
    assert_error(
        rate_countercurrent(GEOMETRY, hot, invalid, LIMITS),
        "invalid_properties",
        "cold annulus",
    );
    invalid = hot;
    invalid.properties.thermal_conductivity_w_per_m_k = f64::NAN;
    assert_error(
        rate_countercurrent(GEOMETRY, invalid, cold, LIMITS),
        "invalid_properties",
        "hot inner tube",
    );
    invalid = cold;
    invalid.inlet.molar_flow_mol_s = 0.0;
    assert_error(
        rate_countercurrent(GEOMETRY, hot, invalid, LIMITS),
        "invalid_flow",
        "cold annulus",
    );
    invalid = hot;
    invalid.properties.domain.min_pressure_pa = 121_000.0;
    assert_error(
        rate_countercurrent(GEOMETRY, invalid, cold, LIMITS),
        "invalid_property_domain",
        "hot inner tube",
    );
    assert_error(
        rate_countercurrent(GEOMETRY, cold, hot, LIMITS),
        "reversed_inlets",
        "inner tube",
    );
}

#[test]
fn rejects_liquid_domain_violations_at_either_end() {
    let (hot, cold) = benchmark_sides();
    let mut invalid = hot;
    invalid.inlet.temperature_k = 303.15_f64.next_up();
    assert_error(
        rate_countercurrent(GEOMETRY, invalid, cold, LIMITS),
        "inlet_outside_domain",
        "hot inner tube",
    );
    invalid = cold;
    invalid.properties.domain.min_pressure_pa = 80_000.0;
    assert_error(
        rate_countercurrent(GEOMETRY, hot, invalid, LIMITS),
        "outlet_outside_domain",
        "cold annulus",
    );
    invalid = hot;
    invalid.properties.domain.min_temperature_k = 300.0;
    assert_error(
        rate_countercurrent(GEOMETRY, invalid, cold, LIMITS),
        "outlet_outside_domain",
        "hot inner tube",
    );
}

#[test]
fn rejects_reynolds_and_prandtl_values_outside_supported_ranges() {
    let (hot, cold) = benchmark_sides();
    for cold_flow in [10.0, 70.0] {
        let mut invalid = cold;
        invalid.inlet.molar_flow_mol_s = cold_flow;
        assert_error(
            rate_countercurrent(GEOMETRY, hot, invalid, LIMITS),
            "outside_correlation_domain",
            "cold annulus",
        );
    }
    for factor in [0.5, 3.0] {
        let mut invalid = hot;
        invalid.properties.thermal_conductivity_w_per_m_k *= factor;
        assert_error(
            rate_countercurrent(GEOMETRY, invalid, cold, LIMITS),
            "outside_correlation_domain",
            "hot inner tube",
        );
    }
}
