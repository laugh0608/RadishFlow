use super::*;
use crate::test_fixtures::binary_hydrocarbon_provider;
use rf_flash::PlaceholderTpFlashSolver;
use rf_unitops::{UnitOperation, UnitOperationInputs, UnitOperationOutputs, UnitOperationServices};
#[test]
fn cooperative_cancellation_at_every_numerical_and_local_stage_never_publishes() {
    struct Stop(EquationStage);
    impl EquationControl for Stop {
        fn checkpoint(&mut self, p: EquationProgress) -> std::ops::ControlFlow<()> {
            if p.stage == self.0 {
                std::ops::ControlFlow::Break(())
            } else {
                std::ops::ControlFlow::Continue(())
            }
        }
    }
    for stage in [
        EquationStage::Assemble,
        EquationStage::StructuralAnalysis,
        EquationStage::LinearEvaluation,
        EquationStage::BracketEndpoint,
        EquationStage::IntervalEvaluation,
        EquationStage::Factorization,
        EquationStage::ConditionSolve,
        EquationStage::FullJacobianCheck,
        EquationStage::FinalUnitVerification,
        EquationStage::GlobalVerification,
        EquationStage::Completing,
    ] {
        let e = EquationOrientedSolver
            .solve_controlled(
                &crate::SolverServices {
                    thermo: &binary_hydrocarbon_provider(),
                    flash_solver: &PlaceholderTpFlashSolver,
                },
                &cross_recycle(0.6, 0.25),
                &inverse_options(),
                &mut Stop(stage),
            )
            .unwrap_err();
        assert_eq!(e.kind, EquationFailureKind::Cancelled, "{stage:?}");
        assert_eq!(e.progress.stage, stage);
    }
}
#[derive(Clone, Copy)]
enum Fault {
    Flow,
    Identity,
    Missing,
    NonFinite,
}
struct Broken {
    inner: Box<dyn UnitOperation>,
    fault: Fault,
}
impl UnitOperation for Broken {
    fn kind(&self) -> rf_unitops::BuiltinUnitKind {
        self.inner.kind()
    }
    fn run(
        &self,
        services: &UnitOperationServices<'_>,
        inputs: &UnitOperationInputs,
    ) -> rf_types::RfResult<UnitOperationOutputs> {
        let original = self.inner.run(services, inputs)?;
        let mut out = UnitOperationOutputs::new();
        let mut a = original.stream("outlet_a").unwrap().clone();
        match self.fault {
            Fault::Flow => a.total_molar_flow_mol_s += 0.01,
            Fault::Identity => a.id = "wrong".into(),
            Fault::NonFinite => a.total_molar_flow_mol_s = f64::NAN,
            Fault::Missing => {}
        }
        if !matches!(self.fault, Fault::Missing) {
            out.insert_material_stream("outlet_a", a);
        }
        out.insert_material_stream("outlet_b", original.stream("outlet_b").unwrap().clone());
        Ok(out)
    }
}
#[test]
fn local_faults_cannot_turn_a_mathematical_candidate_into_success() {
    let flow = cross_recycle(0.6, 0.25);
    let original = flow.clone();
    for fault in [
        Fault::Flow,
        Fault::Identity,
        Fault::Missing,
        Fault::NonFinite,
    ] {
        let mut factory = |u: &rf_model::UnitNode, s: &BTreeMap<StreamId, MaterialStreamState>| {
            let inner = rf_unitops::catalog::instantiate_builtin(u, s)?;
            Ok(if u.id == UnitId::new("sa") {
                Box::new(Broken { inner, fault }) as Box<dyn UnitOperation>
            } else {
                inner
            })
        };
        let e = execution::solve(
            &crate::SolverServices {
                thermo: &binary_hydrocarbon_provider(),
                flash_solver: &PlaceholderTpFlashSolver,
            },
            &flow,
            &inverse_options(),
            &mut Uninterrupted,
            &mut factory,
        )
        .unwrap_err();
        assert_eq!(e.kind, EquationFailureKind::LocalVerification);
        assert!(e.evidence.candidate_values_si.is_some());
        assert!(e.evidence.full_jacobian.is_some());
        assert!(!e.evidence.final_units.is_empty());
        assert!(e.evidence.global_material.is_none());
    }
    assert_eq!(flow, original);
}

struct TerminalDrift {
    inner: Box<dyn UnitOperation>,
    delta: f64,
}
impl UnitOperation for TerminalDrift {
    fn kind(&self) -> rf_unitops::BuiltinUnitKind {
        self.inner.kind()
    }
    fn run(
        &self,
        services: &UnitOperationServices<'_>,
        inputs: &UnitOperationInputs,
    ) -> rf_types::RfResult<UnitOperationOutputs> {
        let original = self.inner.run(services, inputs)?;
        let mut output = UnitOperationOutputs::new();
        output.insert_material_stream("outlet_a", original.stream("outlet_a").unwrap().clone());
        let mut terminal = original.stream("outlet_b").unwrap().clone();
        terminal.total_molar_flow_mol_s += self.delta;
        output.insert_material_stream("outlet_b", terminal);
        Ok(output)
    }
}
#[test]
fn global_absolute_scaled_component_and_scaled_total_checks_have_direct_fault_evidence() {
    // Every actual local call and every individual assembled row passes first.
    // The three independent failures exercise global acceptance only.
    for (fresh, delta, total_only) in [
        (8.0, 1.5e-10, false),
        (0.125, 1e-11, false),
        (0.125, 7e-12, true),
    ] {
        let flow = split_chain(fresh);
        let before = flow.clone();
        let e = execution::solve(
            &crate::SolverServices {
                thermo: &binary_hydrocarbon_provider(),
                flash_solver: &PlaceholderTpFlashSolver,
            },
            &flow,
            &options(),
            &mut Uninterrupted,
            &mut |u, streams| {
                let inner = rf_unitops::catalog::instantiate_builtin(u, streams)?;
                Ok(if u.kind == "splitter" {
                    Box::new(TerminalDrift { inner, delta }) as Box<dyn UnitOperation>
                } else {
                    inner
                })
            },
        )
        .unwrap_err();
        assert_eq!(
            e.kind,
            EquationFailureKind::GlobalVerification,
            "fresh={fresh} delta={delta} error={:?}",
            e.error
        );
        assert_eq!(e.progress.stage, EquationStage::GlobalVerification);
        assert_eq!(e.costs.local_unit_calls, 4);
        assert_eq!(e.evidence.final_units.len(), 4);
        assert!(e.evidence.verified_values_si.is_some());
        assert!(
            e.evidence
                .verified_residuals_mol_s
                .as_ref()
                .unwrap()
                .iter()
                .all(|r| r.abs() <= ABSOLUTE_TOLERANCE_MOL_S
                    && (r / fresh).abs() <= SCALED_TOLERANCE)
        );
        let global = e.evidence.global_material.as_ref().unwrap();
        if total_only {
            assert_eq!(
                e.error.context().diagnostic_code(),
                Some("solver.equation_oriented.global_total")
            );
            assert!(
                global
                    .component_residuals_mol_s
                    .values()
                    .all(|r| r.abs() <= ABSOLUTE_TOLERANCE_MOL_S
                        && (r / fresh).abs() <= SCALED_TOLERANCE)
            );
            assert!((global.total_residual_mol_s / fresh).abs() > SCALED_TOLERANCE);
        } else if fresh > 1.0 {
            assert!(
                global
                    .component_residuals_mol_s
                    .values()
                    .any(|r| r.abs() > ABSOLUTE_TOLERANCE_MOL_S)
            );
            assert!(
                global
                    .component_residuals_mol_s
                    .values()
                    .all(|r| (r / fresh).abs() <= SCALED_TOLERANCE)
            );
        } else {
            assert!(
                global
                    .component_residuals_mol_s
                    .values()
                    .all(|r| r.abs() <= ABSOLUTE_TOLERANCE_MOL_S)
            );
            assert!(
                global
                    .component_residuals_mol_s
                    .values()
                    .any(|r| (r / fresh).abs() > SCALED_TOLERANCE)
            );
        }
        assert_eq!(flow, before);
    }
}
