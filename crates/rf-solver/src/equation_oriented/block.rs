//! An equation block consumes exact external boundaries without creating Feed nodes.
use super::*;
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct EquationBlockInput<'a> {
    pub flowsheet: &'a Flowsheet,
    pub members: &'a BTreeSet<UnitId>,
    pub inputs: &'a BTreeMap<StreamId, MaterialStreamState>,
}
impl EquationBlockInput<'_> {
    pub(super) fn validate(
        &self,
        source: &Flowsheet,
        connections: &[MaterialConnection],
        options: &EquationOptions,
    ) -> EoResult<()> {
        let invalid = |message| {
            error(
                EquationFailureKind::InvalidBinding,
                "block_boundary",
                message,
            )
        };
        if self.members.is_empty() || self.members.iter().any(|id| !source.units.contains_key(id)) {
            return Err(invalid(
                "equation block requires existing, nonempty members",
            ));
        }
        if options.released.is_some() || options.target.is_some() {
            return Err(invalid(
                "equation blocks support fixed-parameter forward material equations only",
            ));
        }
        let incoming: BTreeSet<_> = connections
            .iter()
            .filter(|c| {
                !self.members.contains(&c.source.unit_id)
                    && c.sink
                        .as_ref()
                        .is_some_and(|s| self.members.contains(&s.unit_id))
            })
            .map(|c| c.stream_id.clone())
            .collect();
        if incoming.iter().ne(self.inputs.keys()) {
            return Err(invalid(
                "external inputs must exactly cover incoming engineering connections",
            ));
        }
        for (id, input) in self.inputs {
            if id != &input.id {
                return Err(invalid(
                    "external input identity differs from its connection",
                ));
            }
        }
        Ok(())
    }
}

pub fn assemble_block(
    input: &EquationBlockInput<'_>,
    options: &EquationOptions,
) -> EoResult<EquationProblem> {
    assembly::assemble_scoped(input.flowsheet, options, Some(input), &mut Uninterrupted)
}

pub(crate) fn solve_block_with_factory(
    services: &crate::SolverServices<'_>,
    input: &EquationBlockInput<'_>,
    options: &EquationOptions,
    control: &mut impl EquationControl,
    factory: &mut impl FnMut(
        &rf_model::UnitNode,
        &BTreeMap<StreamId, MaterialStreamState>,
    ) -> rf_types::RfResult<Box<dyn rf_unitops::UnitOperation>>,
) -> Result<EquationResult, Box<EquationFailure>> {
    execution::solve_scoped(
        services,
        input.flowsheet,
        options,
        control,
        factory,
        Some(input),
    )
}
impl EquationOrientedSolver {
    pub fn solve_block_controlled(
        &self,
        services: &crate::SolverServices<'_>,
        input: &EquationBlockInput<'_>,
        options: &EquationOptions,
        control: &mut impl EquationControl,
    ) -> Result<EquationResult, Box<EquationFailure>> {
        execution::solve_scoped(
            services,
            input.flowsheet,
            options,
            control,
            &mut rf_unitops::catalog::instantiate_builtin,
            Some(input),
        )
    }
}
