use super::*;
use std::collections::{BTreeSet, VecDeque};

pub(super) fn analyze(
    problem: &EquationProblem,
    control: &mut impl EquationControl,
) -> EoResult<StructuralReport> {
    let incidence = problem.incidence()?;
    let n = problem.variables.len();
    let m = problem.equations.len();
    let mut owner = vec![None; n];
    for row in 0..m {
        augment(row, &incidence, &mut vec![false; n], &mut owner, control)?;
    }
    let mut matched = vec![None; m];
    for (column, row) in owner.iter().enumerate() {
        if let Some(row) = row {
            matched[*row] = Some(column);
        }
    }
    let unmatched_variables: Vec<_> = owner
        .iter()
        .enumerate()
        .filter_map(|(i, r)| r.is_none().then_some(i))
        .collect();
    let unmatched_equations: Vec<_> = matched
        .iter()
        .enumerate()
        .filter_map(|(i, c)| c.is_none().then_some(i))
        .collect();
    // Alternating reachability from free columns and free rows identifies whole
    // deficient regions; an arbitrary matching does not prescribe a deletion.
    let mut by_column = vec![Vec::new(); n];
    for (r, columns) in incidence.iter().enumerate() {
        for c in columns {
            by_column[*c].push(r);
        }
    }
    let under = region(&unmatched_variables, &by_column, &matched, control)?;
    let over = region(&unmatched_equations, &incidence, &owner, control)?;
    Ok(StructuralReport {
        variable_count: n,
        equation_count: m,
        matching: matched
            .iter()
            .enumerate()
            .filter_map(|(r, c)| {
                c.map(|c| (problem.equations[r].clone(), problem.variables[c].clone()))
            })
            .collect(),
        unmatched_variables: unmatched_variables
            .iter()
            .map(|i| problem.variables[*i].clone())
            .collect(),
        unmatched_equations: unmatched_equations
            .iter()
            .map(|i| problem.equations[*i].clone())
            .collect(),
        underdetermined: StructuralRegion {
            variables: under
                .0
                .iter()
                .map(|i| problem.variables[*i].clone())
                .collect(),
            equations: under
                .1
                .iter()
                .map(|i| problem.equations[*i].clone())
                .collect(),
        },
        overdetermined: StructuralRegion {
            variables: over
                .1
                .iter()
                .map(|i| problem.variables[*i].clone())
                .collect(),
            equations: over
                .0
                .iter()
                .map(|i| problem.equations[*i].clone())
                .collect(),
        },
    })
}
fn augment(
    row: usize,
    edges: &[Vec<usize>],
    visited: &mut [bool],
    owner: &mut [Option<usize>],
    control: &mut impl EquationControl,
) -> EoResult<bool> {
    for &column in &edges[row] {
        let mut p = EquationProgress::at(EquationStage::StructuralAnalysis);
        p.row = Some(row);
        p.column = Some(column);
        checkpoint(control, p)?;
        if visited[column] {
            continue;
        }
        visited[column] = true;
        if owner[column].is_none()
            || augment(
                owner[column].expect("matched column"),
                edges,
                visited,
                owner,
                control,
            )?
        {
            owner[column] = Some(row);
            return Ok(true);
        }
    }
    Ok(false)
}
fn region(
    seeds: &[usize],
    edges: &[Vec<usize>],
    matched_back: &[Option<usize>],
    control: &mut impl EquationControl,
) -> EoResult<(BTreeSet<usize>, BTreeSet<usize>)> {
    let mut a: BTreeSet<_> = seeds.iter().copied().collect();
    let mut b = BTreeSet::new();
    let mut queue: VecDeque<_> = seeds.iter().copied().collect();
    while let Some(i) = queue.pop_front() {
        checkpoint(
            control,
            EquationProgress::at(EquationStage::StructuralAnalysis),
        )?;
        for &j in &edges[i] {
            if matched_back[j] == Some(i) {
                continue;
            }
            b.insert(j);
            if let Some(k) = matched_back[j]
                && a.insert(k)
            {
                queue.push_back(k);
            }
        }
    }
    Ok((a, b))
}
