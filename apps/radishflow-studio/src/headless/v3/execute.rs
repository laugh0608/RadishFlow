use super::{
    actions::*,
    confirmation::Aliases,
    equipment, flow,
    protocol::*,
    query,
    references::{Bindings, Executions},
};
use crate::StudioProjectOpen;
use rf_ui::engineering::editor::{EquipmentAction, EquipmentActionOutcome};
use serde_json::{Value, json};
use std::path::Path;

pub fn file(path: &Path, response: &mut Response) -> Result<(), Failure> {
    let bytes = super::request_bytes(path)?;
    if serde_json::from_slice::<serde_json::Value>(&bytes).map_err(super::invalid_request)?["schema_version"]
        == 6
    {
        response.schema_version = 6;
        return super::super::planned::execute_file::<super::super::v6::Plan>(
            path, &bytes, response,
        );
    }
    if serde_json::from_slice::<serde_json::Value>(&bytes).map_err(super::invalid_request)?["schema_version"]
        == 5
    {
        response.schema_version = 5;
        return super::super::planned::execute_file::<super::super::v5::Plan>(
            path, &bytes, response,
        );
    }
    if serde_json::from_slice::<serde_json::Value>(&bytes).map_err(super::invalid_request)?["schema_version"]
        == 4
    {
        response.schema_version = 4;
        return super::super::v4::execute_file::<super::super::v4::protocol::Plan>(
            path, &bytes, response,
        );
    }
    let request: ExecuteRequest = serde_json::from_slice(&bytes).map_err(super::invalid_request)?;
    debug_assert_eq!(request.schema_version, 3);
    super::runs::validate_resources(&request)?;
    super::validate_ids(request.steps.iter().map(|s| s.id.as_str()))?;
    super::validate_ids(request.queries.iter().map(|q| q.id.as_str()))?;
    if request.project.as_os_str().is_empty() {
        return Err(Failure::new(
            Stage::Request,
            Code::InvalidRequest,
            "project path must not be empty",
        ));
    }
    let (opened, snapshot) = super::load_project(
        &path
            .parent()
            .unwrap_or(Path::new("."))
            .join(&request.project),
    )?;
    let file_hash = snapshot.file_sha256.clone();
    let identity_matches = request.expected_project.document_id == snapshot.document_id;
    let hash_matches = request.expected_project.file_sha256 == file_hash;
    response.project = Some(snapshot);
    if !identity_matches {
        return Err(Failure::new(
            Stage::Project,
            Code::DifferentDocument,
            "document id differs",
        ));
    }
    if !hash_matches {
        return Err(Failure::new(
            Stage::Project,
            Code::ProjectConflict,
            "project file snapshot differs",
        ));
    }
    let StudioProjectOpen::Editable(mut app) = opened else {
        let mut e = Failure::new(
            Stage::Project,
            Code::ReadOnlyProject,
            "unknown equipment capability protects the whole project",
        );
        e.status = Status::Blocked;
        return Err(e);
    };
    let mut refs = Bindings::default();
    let mut executions = Executions::default();
    let mut aliases = Aliases::default();
    super::super::engine::steps(
        &mut app,
        &request.steps,
        response,
        &mut refs,
        |app, refs, index, step| {
            if super::runs::is_run(&step.action) {
                reject_unneeded_confirmation(step)
                    .and_then(|()| {
                        super::runs::execute(
                            app,
                            step,
                            refs,
                            &mut executions,
                            request.flow_resources.as_ref(),
                            path.parent().unwrap_or(Path::new(".")),
                        )
                    })
                    .map(|r| (r.receipt, r.error))
            } else {
                step_action(
                    app,
                    step,
                    refs,
                    &mut aliases,
                    &file_hash,
                    &request.steps[..=index],
                    request.flow_resources.as_ref(),
                )
                .map(|r| (r, None))
            }
        },
    );
    for q in &request.queries {
        query::push_reply(
            q,
            query::editable(&app, &q.query, &refs, &executions),
            response,
        );
    }
    Ok(())
}
pub(in crate::headless) fn step_action(
    app: &mut rf_ui::AppState,
    step: &Step,
    refs: &mut Bindings,
    aliases: &mut Aliases,
    file_hash: &str,
    prefix: &[Step],
    resources: Option<&FlowResources>,
) -> Result<Value, Failure> {
    step_action_review(app, step, refs, aliases, file_hash, prefix, resources, None)
}

#[allow(clippy::too_many_arguments)]
pub(in crate::headless) fn step_action_review(
    app: &mut rf_ui::AppState,
    step: &Step,
    refs: &mut Bindings,
    aliases: &mut Aliases,
    file_hash: &str,
    prefix: &[Step],
    resources: Option<&FlowResources>,
    review_v4: Option<(u32, &Value)>,
) -> Result<Value, Failure> {
    // The entire step is isolated, including the domain's confirmation state.
    let mut staged = app.clone();
    if let Some(flow::FlowReply {
        mut receipt,
        outputs,
    }) = flow::execute(&mut staged, &step.action, refs)?
    {
        reject_unneeded_confirmation(step)?;
        aliases.created(&step.id, app, &staged, &outputs);
        receipt["outputs"] = json!(outputs);
        refs.0.insert(step.id.clone(), outputs);
        *app = staged;
        return Ok(receipt);
    }
    let action = equipment::prepare(&staged, &step.action, refs)?;
    let mut receipt = equipment::apply(&mut staged, action)?;
    let before = receipt
        .target
        .as_ref()
        .and_then(|t| equipment::version(app, t));
    let confirmation = match &receipt.outcome {
        EquipmentActionOutcome::ConfirmationRequired { key, .. } => {
            Some(EquipmentAction::Confirm(key.clone()))
        }
        EquipmentActionOutcome::AdoptionConfirmationRequired { key } => {
            Some(EquipmentAction::ConfirmAdoption(key.clone()))
        }
        _ => None,
    };
    let adoption = staged
        .workspace
        .equipment_editor
        .geometry_adoption_preview()
        .map(|v| v.proposal().clone());
    let mut reviewed = None;
    let mut committed_aliases = None;
    if let Some(confirm) = confirmation {
        let mut proposed = staged.clone();
        let confirmed = equipment::apply(&mut proposed, confirm.clone())?;
        let outputs = equipment::outputs(&proposed, confirmed.target.as_ref());
        let mut proposal_aliases = aliases.clone();
        proposal_aliases.created(&step.id, app, &proposed, &outputs);
        if let Some(adoption) = &adoption {
            proposal_aliases.adoption(&step.id, adoption, &proposed);
        }
        let mut preview = json!({"before":aliases.snapshot(app),"after":proposal_aliases.snapshot(&proposed),"impacts":proposal_aliases.impacts(&confirmed.impacts)});
        if let Some(adoption) = &adoption {
            let candidate = app
                .workspace
                .engineering
                .current_candidate(adoption.source_task())
                .expect("domain validated current candidate");
            preview["candidate"] = aliases.candidate(candidate);
        }
        let digest = if let Some((version, review)) = review_v4 {
            super::confirmation::digest_planned(version, file_hash, review, resources, &preview)?
        } else {
            super::confirmation::digest(file_hash, prefix, resources, &preview)?
        };
        let matches = step
            .confirmation
            .as_ref()
            .is_some_and(|c| c.digest == digest);
        if !matches {
            let code = if step.confirmation.is_some() {
                Code::ConfirmationMismatch
            } else {
                Code::ConfirmationRequired
            };
            let mut e = Failure::new(
                Stage::Step,
                code,
                "review this proposal and replay the request with its digest",
            );
            e.status = Status::ConfirmationRequired;
            e.details = Some(Box::new(
                json!({"confirmation_digest":digest,"preview":preview}),
            ));
            return Err(e);
        }
        receipt = equipment::apply(&mut staged, confirm)?;
        reviewed = Some(digest);
        committed_aliases = Some(proposal_aliases);
    } else {
        reject_unneeded_confirmation(step)?;
    }
    let mut outputs = equipment::outputs(&staged, receipt.target.as_ref());
    let created_sources=adoption.as_ref().map(|proposal| {
        let id=proposal.design_source();
        let record=staged.workspace.engineering.inputs().basis.sources.iter().find(|s|s.id==*id).expect("committed design source");
        outputs.insert(Output::Source,super::references::BoundOutput {id:id.to_string(),revision:record.revision});
        json!({"design":IdentityRef::id(id),"copies":proposal.source_copies().iter().map(|(source,copy)|json!({"source":IdentityRef::id(source),"copy":IdentityRef::id(copy)})).collect::<Vec<_>>()})
    });
    let outcome = match &receipt.outcome {
        EquipmentActionOutcome::Committed => "committed",
        EquipmentActionOutcome::Unchanged => "unchanged",
        EquipmentActionOutcome::SizingChecked { .. }
        | EquipmentActionOutcome::RatingChecked { .. } => "checked",
        _ => unreachable!("confirmation was resolved before publication"),
    };
    let issues = match &receipt.outcome {
        EquipmentActionOutcome::SizingChecked { issues, .. } => {
            Some(issues.iter().map(super::issues::sizing).collect::<Vec<_>>())
        }
        EquipmentActionOutcome::RatingChecked { issues, .. } => {
            Some(issues.iter().map(super::issues::rating).collect::<Vec<_>>())
        }
        _ => None,
    };
    let after = receipt
        .target
        .as_ref()
        .and_then(|t| equipment::version(&staged, t));
    let result = json!({"outcome":outcome,"target":receipt.target.as_ref().map(equipment::object),"before":before,"after":after,"outputs":outputs,"impacts":Aliases::default().impacts(&receipt.impacts),"issues":issues,"confirmed_digest":reviewed,"created_sources":created_sources});
    if let Some(confirmed) = committed_aliases {
        *aliases = confirmed;
    }
    aliases.created(&step.id, app, &staged, &outputs);
    refs.0.insert(step.id.clone(), outputs);
    *app = staged;
    Ok(result)
}
fn reject_unneeded_confirmation(step: &Step) -> Result<(), Failure> {
    if step.confirmation.is_some() {
        return Err(Failure::new(
            Stage::Step,
            Code::InvalidRequest,
            "this action does not require confirmation",
        ));
    }
    Ok(())
}
