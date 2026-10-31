//! Inject a faulty local operation into the private worker to exercise checks
//! that a well-behaved fixed-fraction Splitter cannot naturally violate.
use super::*;
use crate::partitioned::tests::case;
use crate::test_fixtures::binary_hydrocarbon_provider;
use rf_flash::PlaceholderTpFlashSolver;
use rf_unitops::{BuiltinUnitKind, Splitter, StreamTarget, UnitOperationOutputs};
use std::rc::Rc;

#[derive(Clone, Copy)]
enum Fault {
    VerificationDefect,
    MissingOutput,
    WrongIdentity,
    NegativeFlow,
    NonfiniteFlow,
    LostMaterial,
    TemperatureDrift,
}
struct FaultySplitter {
    verifying: Rc<Cell<bool>>,
    fault: Fault,
}
impl UnitOperation for FaultySplitter {
    fn kind(&self) -> BuiltinUnitKind {
        BuiltinUnitKind::Splitter
    }
    fn run(
        &self,
        services: &UnitOperationServices<'_>,
        inputs: &UnitOperationInputs,
    ) -> RfResult<UnitOperationOutputs> {
        let fraction = if matches!(self.fault, Fault::VerificationDefect) && self.verifying.get() {
            0.7
        } else {
            0.6
        };
        let result = Splitter::new(
            StreamTarget::new("s-recycle", "s-recycle"),
            StreamTarget::new("s-bleed", "s-bleed"),
            fraction,
        )
        .run(services, inputs)?;
        let mut output = UnitOperationOutputs::new();
        let mut returned = result.stream("outlet_a").unwrap().clone();
        match self.fault {
            Fault::MissingOutput => {}
            Fault::WrongIdentity => returned.id = "wrong".into(),
            Fault::NegativeFlow => returned.total_molar_flow_mol_s = -1.0,
            Fault::NonfiniteFlow => returned.total_molar_flow_mol_s = f64::NAN,
            Fault::LostMaterial => returned.total_molar_flow_mol_s *= 0.5,
            Fault::TemperatureDrift => returned.temperature_k += 1e-7,
            Fault::VerificationDefect => {}
        }
        if !matches!(self.fault, Fault::MissingOutput) {
            output.insert_material_stream("outlet_a", returned);
        }
        output.insert_material_stream("outlet_b", result.stream("outlet_b").unwrap().clone());
        Ok(output)
    }
}
fn inject(
    fault: Fault,
) -> (
    RunError,
    SolveCosts,
    Option<RecycleBlockReport>,
    Vec<LocalUnitEvaluation>,
    usize,
    PartitionedProgress,
) {
    let (flow, options) = case(0.6);
    let plan = assemble_partitioned_plan(&flow, &options).unwrap();
    let thermo = binary_hydrocarbon_provider();
    let flash = PlaceholderTpFlashSolver;
    let verifying = Rc::new(Cell::new(false));
    let checkpoint_flag = verifying.clone();
    let mut control = move |p: PartitionedProgress| {
        if p.stage == PartitionedStage::VerificationStarted {
            checkpoint_flag.set(true);
        }
        ControlFlow::Continue(())
    };
    let mut trial = TrialExecution {
        plan: &plan,
        services: UnitOperationServices {
            thermo: Some(&thermo),
            flash_solver: Some(&flash),
        },
        operations: BTreeMap::new(),
        control: &mut control,
        streams: BTreeMap::new(),
        blocks: Vec::new(),
        active_recycle: None,
        active_units: Vec::new(),
        costs: SolveCosts::default(),
        block_index: 0,
        progress: PartitionedProgress {
            phase: None,
            stage: PartitionedStage::Initializing,
            block: None,
            unit: None,
        },
        global_scales: BTreeMap::new(),
    };
    assert!(trial.initialize().is_ok());
    trial
        .operations
        .insert("s1".into(), Box::new(FaultySplitter { verifying, fault }));
    let error = trial.execute().expect_err("fault must not publish success");
    (
        error,
        trial.costs,
        trial.active_recycle,
        trial.active_units,
        trial.blocks.len(),
        trial.progress,
    )
}
#[test]
fn failed_final_defect_is_retained_instead_of_reusing_the_candidate() {
    let (error, costs, report, final_units, blocks, progress) = inject(Fault::VerificationDefect);
    assert_eq!(error.kind, PartitionedFailureKind::Verification);
    assert_eq!(progress.phase, Some(EvaluationPhase::Verification));
    assert_eq!(costs.iteration_evaluations, 48);
    assert_eq!(costs.verification_evaluations, 1);
    assert_eq!(blocks, 3);
    assert!(!costs.calls_by_unit.contains_key(&UnitId::new("h1")));
    let report = report.unwrap();
    assert_eq!(report.verification_evaluations, 1);
    assert!(!checks::converged(report.residual_trace.last().unwrap()));
    assert_eq!(final_units.len(), 2);
    assert_eq!(final_units[0].consumed_streams[1], report.assumed);
    assert_eq!(final_units[1].produced_streams[0], report.returned);
}
#[test]
fn faulty_local_outputs_fail_identity_boundary_and_material_checks() {
    for (fault, expected) in [
        (
            Fault::MissingOutput,
            PartitionedFailureKind::LocalEvaluation,
        ),
        (
            Fault::WrongIdentity,
            PartitionedFailureKind::LocalEvaluation,
        ),
        (Fault::NegativeFlow, PartitionedFailureKind::Boundary),
        (Fault::NonfiniteFlow, PartitionedFailureKind::Boundary),
        (Fault::TemperatureDrift, PartitionedFailureKind::Boundary),
        (Fault::LostMaterial, PartitionedFailureKind::Balance),
    ] {
        let (error, costs, _, _, blocks, _) = inject(fault);
        assert_eq!(error.kind, expected);
        assert_eq!(blocks, 3);
        assert_eq!(costs.calls_by_unit[&UnitId::new("s1")], 1);
        assert!(!costs.calls_by_unit.contains_key(&UnitId::new("h1")));
    }
}
