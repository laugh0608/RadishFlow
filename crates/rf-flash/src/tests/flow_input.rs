use super::build_provider;
use crate::{FlashStatus, PlaceholderTpFlashSolver, TpFlashInput, TpFlashSolver};

#[test]
fn rejects_non_finite_and_negative_total_flow() {
    let thermo = build_provider([2.0, 0.5], 100_000.0);
    for flow in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
        let input = TpFlashInput::new("feed", "Feed", 300.0, 100_000.0, flow, vec![0.5, 0.5]);
        let error = PlaceholderTpFlashSolver
            .flash(&thermo, &input)
            .expect_err("invalid flow must fail before reporting convergence");
        assert_eq!(error.code(), rf_types::ErrorCode::InvalidInput);
        assert!(error.message().contains("finite non-negative"));
    }
}

#[test]
fn preserves_zero_and_positive_total_flow() {
    let thermo = build_provider([2.0, 0.5], 100_000.0);
    for flow in [0.0, 1.0, 25.0] {
        let input = TpFlashInput::new("feed", "Feed", 300.0, 100_000.0, flow, vec![0.5, 0.5]);
        let result = PlaceholderTpFlashSolver.flash(&thermo, &input).unwrap();
        assert_eq!(result.status, FlashStatus::Converged);
        assert_eq!(result.stream.total_molar_flow_mol_s, flow);
        let beta = result.vapor_fraction.unwrap();
        assert!((beta - 0.5).abs() < 1e-10);
    }
}
