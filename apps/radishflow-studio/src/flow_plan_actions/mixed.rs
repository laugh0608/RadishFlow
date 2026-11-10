use super::*;
use rf_solver::mixed::*;
pub(super) fn stored(o: &MixedOptions) -> StoredMixedPlan {
    StoredMixedPlan {
        fixed_boundary: StoredFixedBoundary {
            temperature_k: o.boundary.temperature_k,
            pressure_pa: o.boundary.pressure_pa,
        },
        blocks: o
            .blocks
            .iter()
            .map(|b| StoredMixedBlock {
                id: b.id.as_str().into(),
                members: b.members.iter().cloned().collect(),
                method: match &b.method {
                    MixedBlockMethod::Local => StoredMixedBlockMethod::Local,
                    MixedBlockMethod::EquationOriented { .. } => {
                        StoredMixedBlockMethod::EquationOriented {
                            equation_contract_version: 1,
                            scaling: equation_oriented::stored(
                                &b.equation_options(o.boundary).expect("EO block"),
                            )
                            .scaling,
                        }
                    }
                },
            })
            .collect(),
    }
    .canonicalized()
}
pub(super) fn configuration(p: StoredMixedPlan) -> RfResult<MixedOptions> {
    p.validate()?;
    let mut blocks = vec![];
    for b in p.blocks {
        let method = match b.method {
            StoredMixedBlockMethod::Local => MixedBlockMethod::Local,
            StoredMixedBlockMethod::EquationOriented { scaling, .. } => {
                MixedBlockMethod::EquationOriented {
                    scales: match scaling {
                        StoredEquationScaling::EvaluatedFreshTotal => None,
                        StoredEquationScaling::ExplicitEquations {
                            variables,
                            equations,
                        } => Some(rf_solver::equation_oriented::ExplicitEquationScales {
                            variables: variables
                                .into_iter()
                                .map(|s| (equation_oriented::variable(s.binding), s.scale_si))
                                .collect(),
                            equations: equations
                                .into_iter()
                                .map(|s| Ok((equation_oriented::equation(s.binding)?, s.scale_si)))
                                .collect::<RfResult<_>>()?,
                        }),
                    },
                }
            }
        };
        blocks.push(MixedBlock {
            id: MixedBlockId::new(b.id),
            members: b.members.into_iter().collect(),
            method,
        });
    }
    let o = MixedOptions {
        boundary: FixedRecycleBoundary {
            temperature_k: p.fixed_boundary.temperature_k,
            pressure_pa: p.fixed_boundary.pressure_pa,
        },
        blocks,
    };
    o.validate_shape()?;
    Ok(o.canonicalized())
}
