//! Immutable table capture with execution and display-unit guards for output.
use super::*;
use rf_types::units::{ConversionError, DisplayUnitSet, QuantityKind};
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialBalanceTable {
    pub headers: [String; 5],
    pub rows: Vec<[String; 5]>,
    pub total_flow: String,
    pub implementation: &'static str,
    pub revision: u64,
    guard: MaterialBalanceResultGuard,
    units: DisplayUnitSet,
}
impl MaterialBalanceTable {
    pub fn capture(workspace: &WorkspaceState) -> Result<Option<Self>, ConversionError> {
        let MaterialBalanceResultView::Current { solution, origin } =
            workspace.material_balance_result()
        else {
            return Ok(None);
        };
        let units = workspace.project_presentation.display_units().clone();
        let unit = units.unit_for(QuantityKind::MolarFlow);
        let fraction = units.unit_for(QuantityKind::MoleFraction);
        let mut rows = Vec::new();
        for component in solution.values.mole_fractions.keys() {
            let flow = |port| {
                solution
                    .variables
                    .iter()
                    .zip(&solution.point_mol_s)
                    .find(|(v, _)| {
                        v.binding.port == ComponentBalancePort::Mixer(port)
                            && &v.binding.component_id == component
                    })
                    .map(|(_, x)| *x)
                    .expect("assembled component port")
            };
            let label = workspace
                .document
                .flowsheet
                .components
                .get(component)
                .expect("qualified component")
                .name
                .clone();
            rows.push([
                label,
                crate::format_quantity(
                    flow(MixerBalancePort::InletA),
                    QuantityKind::MolarFlow,
                    unit,
                )?,
                crate::format_quantity(
                    flow(MixerBalancePort::InletB),
                    QuantityKind::MolarFlow,
                    unit,
                )?,
                crate::format_quantity(
                    flow(MixerBalancePort::Outlet),
                    QuantityKind::MolarFlow,
                    unit,
                )?,
                crate::format_quantity(
                    solution.values.mole_fractions[component],
                    QuantityKind::MoleFraction,
                    fraction,
                )?,
            ]);
        }
        Ok(Some(Self {
            headers: ["Component", "Inlet A", "Inlet B", "Outlet", "Mole fraction"]
                .map(str::to_owned),
            rows,
            total_flow: crate::format_quantity(
                solution.values.total_molar_flow_mol_s,
                QuantityKind::MolarFlow,
                unit,
            )?,
            implementation: origin.implementation(),
            revision: origin.qualification().input().revision(),
            guard: MaterialBalanceResultGuard::capture(workspace).expect("current result"),
            units,
        }))
    }
    pub fn is_current(&self, workspace: &WorkspaceState) -> bool {
        self.guard.read(workspace).is_some()
            && workspace.project_presentation.display_units() == &self.units
    }
    pub fn tsv(&self, workspace: &WorkspaceState) -> Option<String> {
        if !self.is_current(workspace) {
            return None;
        }
        let quote = |v: &String| {
            if v.contains(['\t', '\n', '\r', '"']) {
                format!("\"{}\"", v.replace('"', "\"\""))
            } else {
                v.clone()
            }
        };
        let mut lines = vec![
            format!(
                "Material balance · revision {} · {}",
                self.revision, self.implementation
            ),
            self.headers
                .iter()
                .map(quote)
                .collect::<Vec<_>>()
                .join("\t"),
        ];
        lines.extend(
            self.rows
                .iter()
                .map(|row| row.iter().map(quote).collect::<Vec<_>>().join("\t")),
        );
        lines.push(format!("Total outlet\t{}", self.total_flow));
        Some(lines.join("\n"))
    }
}
