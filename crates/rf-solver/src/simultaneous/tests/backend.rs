use super::*;
fn linear(a: &[Vec<f64>], b: &[f64]) -> RunResult<dense::LinearSolution> {
    dense::solve(
        a,
        b,
        &mut ModularCosts::default(),
        &mut None,
        &mut |_, _| Ok(()),
    )
}
#[test]
fn pivoted_lu_solves_an_independently_known_system_and_verifies_original_matrix() {
    // [0 2; 1 3] [1,2] = [4,7]; first column requires a row exchange.
    let result = linear(&[vec![0.0, 2.0], vec![1.0, 3.0]], &[4.0, 7.0]).unwrap();
    assert_eq!(result.values, vec![1.0, 2.0]);
    assert_eq!(result.evidence.row_swaps, vec![(0, 1)]);
    assert_eq!(result.evidence.backward_error, Some(0.0));
    // ||A||inf=4, ||A^-1||inf=2.5, reciprocal condition=0.1.
    close(result.evidence.rcond_indicator.unwrap(), 0.1, 1e-15);
    assert_eq!(linear(&[vec![2.0]], &[0.0]).unwrap().values, vec![0.0]);
}
#[test]
fn invalid_singular_overflow_and_excessive_dimensions_never_return_a_solution() {
    for (a, b) in [
        (vec![], vec![]),
        (vec![vec![1.0]], vec![1.0, 2.0]),
        (vec![vec![f64::NAN]], vec![1.0]),
        (vec![vec![1.0]], vec![f64::INFINITY]),
        (
            vec![vec![f64::MAX, f64::MAX], vec![1.0, 2.0]],
            vec![1.0, 1.0],
        ),
    ] {
        assert!(linear(&a, &b).is_err());
    }
    assert_eq!(
        linear(&[vec![0.0]], &[1.0]).unwrap_err().kind,
        ModularFailureKind::Singular
    );
    assert_eq!(
        linear(&[vec![1.0, 1.0], vec![2.0, 2.0]], &[1.0, 2.0])
            .unwrap_err()
            .kind,
        ModularFailureKind::Singular
    );
    let n = MAX_UNKNOWNS;
    let a: Vec<_> = (0..n)
        .map(|i| (0..n).map(|j| f64::from(i == j)).collect())
        .collect();
    assert_eq!(linear(&a, &vec![1.0; n]).unwrap().values, vec![1.0; n]);
    assert!(linear(&vec![vec![0.0; n + 1]; n + 1], &vec![1.0; n + 1]).is_err());
    // Row norm remains finite; elimination overflows instead.
    assert!(linear(&[vec![1e308, 1e307], vec![1e308, -1e307]], &[1e308, 1e308]).is_err());
}
#[test]
fn condition_refusal_keeps_numeric_evidence_and_linear_safe_points_cancel() {
    let a = [vec![1.0, 0.0], vec![0.0, 1e-13]];
    let mut evidence = None;
    let mut counts = ModularCosts::default();
    let error = dense::solve(&a, &[1.0, 1.0], &mut counts, &mut evidence, &mut |_, _| {
        Ok(())
    })
    .unwrap_err();
    assert_eq!(error.kind, ModularFailureKind::IllConditioned);
    assert_eq!(evidence.unwrap().rcond_indicator, Some(1e-13));
    assert_eq!(counts.triangular_solves, 2);
    for stage in [
        ModularStage::Factorization,
        ModularStage::ConditionSolve,
        ModularStage::LinearVerification,
    ] {
        let err = dense::solve(
            &[vec![1.0]],
            &[1.0],
            &mut ModularCosts::default(),
            &mut None,
            &mut |s, _| {
                if s == stage {
                    Err(RunError {
                        kind: ModularFailureKind::Cancelled,
                        error: failure("cancelled", "test"),
                    })
                } else {
                    Ok(())
                }
            },
        )
        .unwrap_err();
        assert_eq!(err.kind, ModularFailureKind::Cancelled);
    }
}
