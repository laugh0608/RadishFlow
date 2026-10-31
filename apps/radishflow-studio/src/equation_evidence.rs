//! Human-readable captured EO evidence; all quantities honor display units.
use crate::result_evidence_text::{EvidenceLines, Projection, diagnostic_number};
use rf_solver::equation_oriented::*;
use rf_types::units::{DisplayUnitSet, QuantityKind};
fn linear(p: &mut Projection<'_>, label: &str, l: Option<&LinearEvidence>) {
    match l {
        Some(l) => p.output.lines.push(format!(
            "{label}: n={} · norm={} · pivot={} · rcond={} · backward={}",
            l.dimension,
            diagnostic_number(Some(l.matrix_inf_norm)),
            diagnostic_number(Some(l.pivot_threshold)),
            diagnostic_number(l.rcond_indicator),
            diagnostic_number(l.backward_error)
        )),
        None => p
            .output
            .lines
            .push(format!("{label}: {}", diagnostic_number(None))),
    }
}
fn evidence(p: &mut Projection<'_>, e: &EquationEvidence) {
    if let Some(d) = &e.definition {
        p.output.lines.push(format!("Variables {} · equations {} · matched {} · structural matching does not prove numerical solvability",d.variables.len(),d.equations.len(),d.structure.matching.len()));
        for (i, b) in d.variables.iter().enumerate() {
            let kind = if matches!(b, VariableBinding::Parameter(_)) {
                QuantityKind::DimensionlessRatio
            } else {
                QuantityKind::MolarFlow
            };
            let value = e
                .verified_values_si
                .as_ref()
                .or(e.candidate_values_si.as_ref())
                .and_then(|v| v.get(i))
                .map(|v| p.quantity("EO variable", kind, *v))
                .unwrap_or_else(|| diagnostic_number(None));
            let scale = p.quantity("EO variable scale", kind, d.variable_scales_si[i]);
            p.output
                .lines
                .push(format!("{b:?}: {value} · scale {scale}"));
        }
        for (i, b) in d.equations.iter().enumerate() {
            let residual = e
                .verified_residuals_mol_s
                .as_ref()
                .or(e.candidate_residuals_mol_s.as_ref())
                .and_then(|r| r.get(i))
                .copied();
            let text = residual
                .map(|v| p.flow("EO residual", v))
                .unwrap_or_else(|| diagnostic_number(None));
            let scale = p.flow("EO residual scale", d.residual_scales_mol_s[i]);
            p.output.lines.push(format!(
                "{b:?}: r={text} · scale {scale} · scaled {}",
                diagnostic_number(residual.map(|r| r / d.residual_scales_mol_s[i]))
            ));
        }
        for (binding, value) in &d.fixed_split_fractions {
            p.output
                .lines
                .push(format!("Fixed {binding:?}: {value:.12}"));
        }
    }
    if let Some(s) = &e.solved_parameter {
        p.output.lines.push(format!(
            "{} / outlet_a_split_fraction: stored {} · solved {:.12} · interval [{:.12}, {:.12}]",
            s.binding.unit,
            diagnostic_number(s.stored_value),
            s.value,
            s.interval.0,
            s.interval.1
        ));
    }
    if let Some(t) = &e.verified_target {
        let target = p.flow(t.target.source.stream.as_str(), t.target.value_mol_s);
        let actual = p.flow(t.target.source.stream.as_str(), t.actual_mol_s);
        let residual = p.flow(t.target.source.stream.as_str(), t.residual_mol_s);
        p.output.lines.push(format!(
            "Product {}: target {target} · actual {actual} · residual {residual} · scaled {:.9e}",
            t.target.source.stream, t.scaled_residual
        ));
    }
    linear(p, "Full Jacobian", e.full_jacobian.as_ref());
    for (i, s) in e.conditional_evaluations.iter().enumerate() {
        let r = s
            .target_residual_mol_s
            .map(|v| p.flow("Target residual", v))
            .unwrap_or_else(|| diagnostic_number(None));
        p.output.lines.push(format!(
            "Conditional sample {} · alpha {} · interval {:?} · completed {} · target residual {r}",
            i + 1,
            diagnostic_number(s.fraction),
            s.interval,
            s.completed
        ));
        linear(p, "Conditional material system", s.linear.as_ref());
    }
    p.local_units(&e.final_units);
    for (port, source) in &e.zero_flow_sources {
        p.output
            .lines
            .push(format!("Zero composition anchor: {port:?} ← {source:?}"));
    }
    if let Some(g) = &e.global_material {
        for (id, r) in &g.component_residuals_mol_s {
            let value = p.flow(id.as_str(), *r);
            p.output
                .lines
                .push(format!("Global material residual {id}: {value}"));
        }
        let value = p.flow("Global total", g.total_residual_mol_s);
        p.output
            .lines
            .push(format!("Global total residual: {value}"));
    }
}
fn costs(c: &EquationCosts) -> Vec<String> {
    vec![
        format!(
            "assemblies {} · structural analyses {} · conditional evaluations {} · full Jacobian evaluations {} · target evaluations {} · interval updates {}",
            c.assemblies,
            c.structural_analyses,
            c.conditional_linear_evaluations,
            c.full_jacobian_evaluations,
            c.target_evaluations,
            c.interval_updates
        ),
        format!(
            "factorizations {} · triangular solves {} · operation constructions {} · actual local calls {} · thermo {} · flash {} · elapsed {:.6} s",
            c.factorizations,
            c.triangular_solves,
            c.operation_constructions,
            c.local_unit_calls,
            c.thermo_calls,
            c.flash_calls,
            c.elapsed.as_secs_f64()
        ),
    ]
}
pub fn result_lines(r: &EquationResult, units: &DisplayUnitSet) -> EvidenceLines {
    let mut p = Projection::new(units);
    p.output.lines.push(format!(
        "Current EO condition: {} · fixed-T/P material closure",
        r.backend
    ));
    evidence(&mut p, &r.evidence);
    p.output.lines.extend(costs(&r.costs));
    p.output
}
pub fn failure_lines(f: &EquationFailure, units: &DisplayUnitSet) -> EvidenceLines {
    let mut p = Projection::new(units);
    p.output.lines.push(format!(
        "EO diagnostic only: {:?} · {} · stage {:?}",
        f.kind, f.error, f.progress
    ));
    evidence(&mut p, &f.evidence);
    p.output.lines.extend(costs(&f.costs));
    p.output
}
