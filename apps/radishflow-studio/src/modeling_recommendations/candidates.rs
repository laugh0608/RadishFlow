use super::*;
use crate::modeling_actions::{connection_target, current_connections};
use crate::modeling_specifications::{ModelingSpecificationReport, SpecificationProblem};
use rf_ui::composition_commands::preview_composition_normalization;
use rf_ui::{AppState, CanvasSuggestedStreamBinding};

pub(super) fn recommendations(
    app: &AppState,
    report: &ModelingSpecificationReport,
) -> Vec<ModelingRecommendation> {
    let document = &app.workspace.document;
    let mut result = Vec::new();
    let make = |rule, target, related_problems, action| ModelingRecommendation {
        key: RecommendationKey {
            document_id: report.identity.document_id.clone(),
            session: report.identity.session,
            rule,
            rule_version: RECOMMENDATION_RULE_VERSION,
            target,
        },
        identity: report.identity.clone(),
        related_problems,
        action,
    };
    for problem in &report.problems {
        let key = problem.key(&report.identity);
        for target in problem
            .targets()
            .into_iter()
            .filter(|target| target.exists(document))
        {
            result.push(make(
                RecommendationRule::LocateInput,
                RecommendationTarget::Navigation {
                    problem: Box::new(key.clone()),
                    target: target.clone(),
                },
                vec![key.clone()],
                RecommendedAction::Locate(target),
            ));
        }
        if let SpecificationProblem::UnnormalizedComposition { stream_id, .. } = problem
            && let Ok(preview) = preview_composition_normalization(document, stream_id)
        {
            result.push(make(
                RecommendationRule::NormalizeComposition,
                RecommendationTarget::Composition(stream_id.clone()),
                vec![key],
                RecommendedAction::NormalizeComposition(preview),
            ));
        }
    }
    for connection in current_connections(app) {
        // Local rules enumerate candidates, not validation authority. Do not run
        // whole-flowsheet validation: missing unrelated ports are expected here.
        if app
            .validate_acyclic_material_connection(&connection)
            .is_err()
        {
            continue;
        }
        let target = connection_target(&connection);
        let related = report.problems.iter().filter(|problem| {
            matches!(problem, SpecificationProblem::Topology(_)) && problem.targets().iter().any(|p| {
                matches!(p, SpecificationTarget::Port(port)
                    if (port.unit_id == target.source.unit_id && port.port_name == target.source.port)
                    || target.sink.as_ref().is_some_and(|sink| port.unit_id == sink.unit_id && port.port_name == sink.port))
            })
        }).map(|problem| problem.key(&report.identity)).collect();
        let rule = match &connection.stream {
            CanvasSuggestedStreamBinding::Create { .. } => RecommendationRule::CreateOutletStream,
            CanvasSuggestedStreamBinding::Existing { .. } => {
                RecommendationRule::ConnectMaterialStream
            }
        };
        result.push(make(
            rule,
            RecommendationTarget::Connection(target),
            related,
            RecommendedAction::MaterialConnection(connection),
        ));
    }
    result
}
