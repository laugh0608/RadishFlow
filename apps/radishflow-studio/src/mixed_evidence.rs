//! Captured mixed evidence with shared display-unit conversion.
use crate::result_evidence_text::{EvidenceLines, Projection};
use rf_solver::mixed::*;
use rf_types::units::DisplayUnitSet;
fn blocks(p: &mut Projection<'_>, blocks: &[MixedBlockReport], units: &DisplayUnitSet) {
    for b in blocks {
        p.output.lines.push(format!(
            "Block {} · members {} · {:.6} s",
            b.id.as_str(),
            b.members
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" + "),
            b.elapsed.as_secs_f64()
        ));
        match &b.evidence {
            MixedBlockEvidence::Local(u) => p.local_units(std::slice::from_ref(u)),
            MixedBlockEvidence::EquationOriented(r) => {
                let e = crate::equation_evidence::result_lines(r, units);
                p.output.lines.extend(e.lines);
                p.output.errors.extend(e.errors);
            }
        }
    }
}
fn boundaries(p: &mut Projection<'_>, values: &[MixedBoundaryEvidence]) {
    for b in values {
        p.output.lines.push(format!(
            "Boundary {}: {}.{} → {}",
            b.connection.stream_id,
            b.connection.source.unit_id,
            b.connection.source.port_name,
            b.connection
                .sink
                .as_ref()
                .map(|s| format!("{}.{}", s.unit_id, s.port_name))
                .unwrap_or_else(|| "terminal".into())
        ));
        p.state("Produced", &b.producer);
        p.state("Consumed", &b.consumed);
        for (c, r) in &b.residuals_mol_s {
            let value = p.flow(c.as_str(), *r);
            p.output.lines.push(format!(
                "Residual {c}: {value}; scaled {:.9e}",
                b.scaled_residuals[c]
            ));
        }
    }
}
fn costs(p: &mut Projection<'_>, c: &MixedCosts) {
    p.output.lines.push(format!(
        "Actual calls: local {}; thermo {}; flash {}; construction {}; total {:.6} s",
        c.local_unit_calls,
        c.thermo_calls,
        c.flash_calls,
        c.operation_constructions,
        c.elapsed.as_secs_f64()
    ));
}
pub fn block_lines(b: &MixedBlockReport, units: &DisplayUnitSet) -> EvidenceLines {
    let mut p = Projection::new(units);
    blocks(&mut p, std::slice::from_ref(b), units);
    p.output
}
pub fn boundary_lines(r: &MixedResult, units: &DisplayUnitSet) -> EvidenceLines {
    let mut p = Projection::new(units);
    boundaries(&mut p, &r.boundaries);
    for (c, r) in &r.global_material.component_residuals_mol_s {
        let value = p.flow(c.as_str(), *r);
        p.output
            .lines
            .push(format!("Global material residual {c}: {value}"));
    }
    costs(&mut p, &r.costs);
    p.output
}
pub fn result_lines(r: &MixedResult, units: &DisplayUnitSet) -> EvidenceLines {
    let mut p = Projection::new(units);
    blocks(&mut p, &r.blocks, units);
    boundaries(&mut p, &r.boundaries);
    for (c, r) in &r.global_material.component_residuals_mol_s {
        let value = p.flow(c.as_str(), *r);
        p.output
            .lines
            .push(format!("Global material residual {c}: {value}"));
    }
    costs(&mut p, &r.costs);
    p.output
}
pub fn failure_lines(f: &MixedFailure, units: &DisplayUnitSet) -> EvidenceLines {
    let mut p = Projection::new(units);
    p.output.lines.push(format!(
        "Mixed failure {:?}: {} · {:?}",
        f.kind, f.error, f.progress
    ));
    blocks(&mut p, &f.completed_blocks, units);
    boundaries(&mut p, &f.boundaries);
    p.local_units(&f.active_units);
    if let Some(e) = &f.equation_failure {
        let e = crate::equation_evidence::failure_lines(e, units);
        p.output.lines.extend(e.lines);
        p.output.errors.extend(e.errors);
    }
    costs(&mut p, &f.costs);
    p.output
}
