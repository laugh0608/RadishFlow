//! Bounded dense LU. Original matrix remains separate for backward verification.
use super::*;
#[derive(Debug)]
pub(super) struct LinearSolution {
    pub values: Vec<f64>,
    pub evidence: LinearEvidence,
}
pub(super) fn solve(
    a: &[Vec<f64>],
    b: &[f64],
    costs: &mut ModularCosts,
    evidence: &mut Option<LinearEvidence>,
    checkpoint: &mut impl FnMut(ModularStage, usize) -> RunResult<()>,
) -> RunResult<LinearSolution> {
    let n = b.len();
    let invalid = |code, msg| RunError {
        kind: ModularFailureKind::LinearVerification,
        error: failure(code, msg),
    };
    if n == 0
        || n > MAX_UNKNOWNS
        || a.len() != n
        || a.iter()
            .any(|r| r.len() != n || r.iter().any(|v| !v.is_finite()))
        || b.iter().any(|v| !v.is_finite())
    {
        return Err(invalid(
            "linear_input",
            "invalid dense dimensions or nonfinite inputs",
        ));
    }
    let norm = a
        .iter()
        .map(|row| row.iter().map(|x| x.abs()).sum::<f64>())
        .fold(0.0, f64::max);
    if !norm.is_finite() {
        return Err(invalid("linear_overflow", "matrix norm overflow"));
    }
    let threshold = 64.0 * n as f64 * f64::EPSILON * norm;
    *evidence = Some(LinearEvidence {
        dimension: n,
        matrix_inf_norm: norm,
        pivot_threshold: threshold,
        row_swaps: Vec::new(),
        rcond_indicator: None,
        backward_error: None,
    });
    let mut lu = a.to_vec();
    let mut swaps = Vec::new();
    costs.factorizations += 1;
    for k in 0..n {
        checkpoint(ModularStage::Factorization, k)?;
        let pivot = (k..n)
            .max_by(|&i, &j| {
                lu[i][k]
                    .abs()
                    .total_cmp(&lu[j][k].abs())
                    .then_with(|| j.cmp(&i))
            })
            .expect("nonempty column");
        if lu[pivot][k].abs() <= threshold || !lu[pivot][k].is_finite() {
            return Err(RunError {
                kind: ModularFailureKind::Singular,
                error: failure("numerical_singular", "pivot below frozen scaled threshold"),
            });
        }
        if pivot != k {
            lu.swap(k, pivot);
            swaps.push((k, pivot));
            evidence.as_mut().expect("initialized").row_swaps = swaps.clone();
        }
        for i in k + 1..n {
            lu[i][k] /= lu[k][k];
            for j in k + 1..n {
                lu[i][j] -= lu[i][k] * lu[k][j];
            }
        }
        if lu.iter().flatten().any(|x| !x.is_finite()) {
            return Err(invalid("linear_overflow", "nonfinite LU arithmetic"));
        }
    }
    let mut inverse_rows = vec![0.0; n];
    for j in 0..n {
        checkpoint(ModularStage::ConditionSolve, j)?;
        let mut rhs = vec![0.0; n];
        rhs[j] = 1.0;
        costs.triangular_solves += 1;
        let col = triangular(&lu, &swaps, rhs)?;
        for i in 0..n {
            inverse_rows[i] += col[i].abs();
        }
    }
    let inverse_norm = inverse_rows.into_iter().fold(0.0, f64::max);
    let condition = norm * inverse_norm;
    let rcond = 1.0 / condition;
    evidence.as_mut().expect("initialized").rcond_indicator = Some(rcond);
    if !condition.is_finite() || !rcond.is_finite() || rcond <= 1e-12 {
        return Err(RunError {
            kind: ModularFailureKind::IllConditioned,
            error: failure(
                "ill_conditioned",
                format!("computed reciprocal condition indicator {rcond} is below acceptance"),
            ),
        });
    }
    checkpoint(ModularStage::LinearVerification, 0)?;
    costs.triangular_solves += 1;
    let x = triangular(&lu, &swaps, b.to_vec())?;
    let residual = a
        .iter()
        .zip(b)
        .map(|(row, &b)| (row.iter().zip(&x).map(|(a, x)| a * x).sum::<f64>() - b).abs())
        .fold(0.0, f64::max);
    let denominator = norm * x.iter().map(|x| x.abs()).fold(0.0, f64::max)
        + b.iter().map(|x| x.abs()).fold(0.0, f64::max);
    let eta = if residual == 0.0 && denominator == 0.0 {
        0.0
    } else {
        residual / denominator
    };
    evidence.as_mut().expect("initialized").backward_error = Some(eta);
    if !residual.is_finite()
        || !denominator.is_finite()
        || !eta.is_finite()
        || eta > 64.0 * n as f64 * f64::EPSILON
    {
        return Err(invalid(
            "linear_backward_error",
            "original matrix backward verification failed",
        ));
    }
    Ok(LinearSolution {
        values: x,
        evidence: evidence.clone().expect("initialized"),
    })
}
fn triangular(lu: &[Vec<f64>], swaps: &[(usize, usize)], mut x: Vec<f64>) -> RunResult<Vec<f64>> {
    for &(i, j) in swaps {
        x.swap(i, j);
    }
    for i in 0..x.len() {
        for j in 0..i {
            x[i] -= lu[i][j] * x[j];
        }
    }
    for i in (0..x.len()).rev() {
        for j in i + 1..x.len() {
            x[i] -= lu[i][j] * x[j];
        }
        x[i] /= lu[i][i];
    }
    if x.iter().any(|x| !x.is_finite()) {
        return Err(RunError {
            kind: ModularFailureKind::LinearVerification,
            error: failure("linear_overflow", "nonfinite triangular solution"),
        });
    }
    Ok(x)
}
