//! QS2 mapping onto the shared bounded dense algebra; public evidence is unchanged.
use super::*;
use crate::dense_linear::{self, DenseCosts, DenseError, DenseFailureKind, DenseStage};
pub(super) use dense_linear::LinearSolution;
pub(super) fn solve(
    a: &[Vec<f64>],
    b: &[f64],
    costs: &mut ModularCosts,
    evidence: &mut Option<LinearEvidence>,
    checkpoint: &mut impl FnMut(ModularStage, usize) -> RunResult<()>,
) -> RunResult<LinearSolution> {
    let mut counts = DenseCosts::default();
    let result = dense_linear::solve(a, b, &mut counts, evidence, &mut |stage, index| {
        checkpoint(
            match stage {
                DenseStage::Factorization => ModularStage::Factorization,
                DenseStage::ConditionSolve => ModularStage::ConditionSolve,
                DenseStage::LinearVerification => ModularStage::LinearVerification,
            },
            index,
        )
    });
    costs.factorizations += counts.factorizations;
    costs.triangular_solves += counts.triangular_solves;
    result.map_err(|e| match e {
        DenseError::Interrupted(e) => e,
        DenseError::Numerical(e) => RunError {
            kind: match e.kind {
                DenseFailureKind::Singular => ModularFailureKind::Singular,
                DenseFailureKind::IllConditioned => ModularFailureKind::IllConditioned,
                DenseFailureKind::Verification => ModularFailureKind::LinearVerification,
            },
            error: failure(e.code, e.message),
        },
    })
}
