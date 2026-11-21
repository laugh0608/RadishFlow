use super::{StoredFlowPlan, StoredProjectV4};
use rf_model::equipment::{ConcentricTubeFlowAssociation, ConcentricTubeRatingTask};
use rf_types::{RfError, RfResult};

fn associations(
    project: &StoredProjectV4,
) -> impl Iterator<
    Item = (
        usize,
        &ConcentricTubeRatingTask,
        &ConcentricTubeFlowAssociation,
    ),
> {
    project.engineering.iter().flat_map(|inputs| {
        inputs.tasks.iter().enumerate().filter_map(|(index, task)| {
            let task = task.as_concentric_tube_rating()?;
            Some((index, task, task.case.flow_association.as_ref()?))
        })
    })
}

pub(super) fn validate(project: &StoredProjectV4) -> RfResult<()> {
    for (index, _, association) in associations(project) {
        let error = |path: &str, message: &str| {
            RfError::invalid_input(format!(
                "engineering.tasks[{index}].case.flowAssociation{path}: {message}"
            ))
            .with_diagnostic_code("store.project.h3_flow_association")
        };
        if !matches!(project.steady_state_plan, StoredFlowPlan::Acyclic) {
            return Err(error("", "requires an acyclic flow plan"));
        }
        let identities = project
            .object_identities
            .as_ref()
            .ok_or_else(|| error("", "requires saved objectIdentities"))?;
        if association.hot.id == association.cold.id {
            return Err(error(
                ".cold.streamId",
                "hot and cold streams must be distinct",
            ));
        }
        for (side, reference) in [("hot", &association.hot), ("cold", &association.cold)] {
            if !project
                .document
                .flowsheet
                .streams
                .contains_key(&reference.id)
            {
                return Err(error(
                    &format!(".{side}.streamId"),
                    "stream must exist in the project",
                ));
            }
            if identities.streams.get(&reference.id) != Some(&reference.creation_id) {
                return Err(error(
                    &format!(".{side}.creationId"),
                    "does not match the saved stream identity",
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn check_overwrite(old: &StoredProjectV4, new: &StoredProjectV4) -> RfResult<()> {
    for (_, old, _) in associations(old) {
        if !associations(new).any(|(_, next, _)| next.id == old.id && next.case.id == old.case.id) {
            return Err(RfError::invalid_input(format!(
                "project target contains a saved flow association for task {} / case {}",
                old.id, old.case.id,
            ))
            .with_diagnostic_code("store.project.h3_flow_association_removal"));
        }
    }
    Ok(())
}
