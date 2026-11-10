//! Session drafts for numerical settings. Applying uses the application's formal action.
use super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialBalanceSetting {
    InletAScale,
    InletBScale,
    OutletScale,
    AbsoluteTolerance,
    ScaledTolerance,
}
impl MaterialBalanceSetting {
    pub const ALL: [Self; 5] = [
        Self::InletAScale,
        Self::InletBScale,
        Self::OutletScale,
        Self::AbsoluteTolerance,
        Self::ScaledTolerance,
    ];
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialBalanceDraftError {
    pub field: MaterialBalanceSetting,
}
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialBalanceDraft {
    session: FlowRunQualification,
    pub mixer_id: rf_types::UnitId,
    pub text: [String; 5],
}
impl MaterialBalanceDraft {
    pub fn from_plan(workspace: &WorkspaceState, plan: &MaterialBalancePlan) -> Self {
        let s = plan.specification.scales;
        Self {
            session: workspace.flow_run_qualification(),
            mixer_id: plan.specification.mixer_id.clone(),
            text: [
                s.inlet_a_mol_s,
                s.inlet_b_mol_s,
                s.outlet_mol_s,
                plan.options.absolute_tolerance_mol_s,
                plan.options.scaled_tolerance,
            ]
            .map(|v| v.to_string()),
        }
    }
    pub fn belongs_to(&self, workspace: &WorkspaceState) -> bool {
        self.session
            .same_session(&workspace.flow_run_qualification())
    }
    pub fn plan(&self) -> Result<MaterialBalancePlan, MaterialBalanceDraftError> {
        let mut values = [0.0; 5];
        for (i, field) in MaterialBalanceSetting::ALL.into_iter().enumerate() {
            let value = self.text[i]
                .trim()
                .parse::<f64>()
                .map_err(|_| MaterialBalanceDraftError { field })?;
            if !value.is_finite() || value <= 0.0 {
                return Err(MaterialBalanceDraftError { field });
            }
            values[i] = value;
        }
        Ok(MaterialBalancePlan {
            specification: MixerBalanceSpecification::new(
                self.mixer_id.clone(),
                MixerBalanceScales {
                    inlet_a_mol_s: values[0],
                    inlet_b_mol_s: values[1],
                    outlet_mol_s: values[2],
                },
            ),
            options: MaterialBalanceOptions {
                algorithm: MaterialBalanceAlgorithm::LowerTriangularSubstitution,
                absolute_tolerance_mol_s: values[3],
                scaled_tolerance: values[4],
            },
        })
    }
    pub fn is_applied(&self, workspace: &WorkspaceState) -> bool {
        self.belongs_to(workspace)
            && self
                .plan()
                .is_ok_and(|plan| workspace.material_balance_plan() == Some(&plan))
    }
}
