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
