use super::*;
use crate::engineering::{
    EngineeringCommand, EngineeringInputs, EngineeringSession, EquipmentResultStatus,
};
use crate::variable_browser::ActionExecution;
use crate::{AppState, DocumentMetadata, FlowsheetDocument};
use rf_model::engineering::BindingOrigin;
use rf_model::equipment::{SeparatorGeometryInput, SeparatorRatingInput};
use rf_model::{Flowsheet, MaterialStreamState};
use std::time::UNIX_EPOCH;

fn app() -> AppState {
    let mut flow = Flowsheet::new("query");
    flow.insert_stream(MaterialStreamState::new("feed", "进料"))
        .unwrap();
    let mut app = AppState::new(FlowsheetDocument::new(
        flow,
        DocumentMetadata::new("query", "Query", UNIX_EPOCH),
    ));
    let rf_store::project_v3::ProjectV3Read::Editable(p) = rf_store::project_v3::parse(
        include_str!("../../../rf-store/tests/fixtures/project-v3-rating.json"),
    )
    .unwrap() else {
        panic!("editable fixture")
    };
    let e = p.engineering.unwrap();
    app.workspace.engineering = EngineeringSession::from_inputs(
        "query".into(),
        EngineeringInputs {
            basis: e.basis,
            tasks: e.tasks,
        },
    )
    .unwrap();
    app
}
fn member() -> EquipmentCaseKey {
    EquipmentCaseKey {
        task_id: "rating-1".into(),
        case_id: "rating-case-1".into(),
    }
}

#[test]
fn discovery_is_scoped_typed_and_read_only() {
    let app = app();
    let before = app.clone();
    let q = ProjectQuery::new(&app.workspace);
    let all = q.objects();
    assert_eq!(all.len(), 8); // stream, task, case, geometry, two basis and two sources
    for object in &all {
        q.record(&object.reference).unwrap();
        q.fields(&object.reference).unwrap();
        q.actions(Some(&object.reference), ActionExecution::Background)
            .unwrap();
    }
    let case = q.reference(ProjectObjectId::Case(member()));
    assert_eq!(
        q.object(&case).unwrap().parent,
        Some(q.reference(ProjectObjectId::Equipment("rating-1".into())))
    );
    assert_eq!(q.search("RATING-CASE-1").len(), 1);
    assert_eq!(
        q.equipment_result(&case).unwrap().status(),
        EquipmentResultStatus::Missing
    );
    let mut foreign = case.clone();
    foreign.document = "other".into();
    assert_eq!(
        q.record(&foreign).unwrap_err(),
        ProjectQueryError::DifferentDocument
    );
    let missing = q.reference(ProjectObjectId::Case(EquipmentCaseKey {
        task_id: "other".into(),
        ..member()
    }));
    assert!(matches!(
        q.record(&missing),
        Err(ProjectQueryError::ObjectMissing(_))
    ));
    assert!(matches!(
        q.read_field(&case, &ProjectField::BasisValue),
        Err(ProjectQueryError::FieldMissing(_))
    ));
    assert_eq!(app, before);
}

#[test]
fn fields_resolve_basis_and_override_in_si_and_preserve_absence() {
    let mut app = app();
    let mut case = app
        .workspace
        .engineering
        .inputs()
        .rating_task(&member().task_id)
        .unwrap()
        .cases[0]
        .clone();
    case.inputs.remove(&SeparatorRatingInput::GasDensity);
    app.workspace
        .engineering
        .apply(vec![EngineeringCommand::UpdateRatingCase {
            task: member().task_id,
            case: Box::new(case),
        }])
        .unwrap();
    let q = ProjectQuery::new(&app.workspace);
    let case = q.reference(ProjectObjectId::Case(member()));
    let value = q
        .read_field(
            &case,
            &ProjectField::RatingInput(SeparatorRatingInput::RequiredWorkingTime),
        )
        .unwrap();
    assert!(
        matches!(value.value, ProjectFieldValue::Engineering(EngineeringQuantityRead { state: EngineeringQuantityState::Value { value_si: 120.0, origin: BindingOrigin::Basis(b), source, .. }, .. }) if b.id.as_str() == "basis-time" && source.revision == 3)
    );
    let geometry = q.reference(ProjectObjectId::Geometry(member().task_id));
    let value = q
        .read_field(
            &geometry,
            &ProjectField::GeometryInput(SeparatorGeometryInput::InternalDiameter),
        )
        .unwrap();
    assert!(
        matches!(value.value, ProjectFieldValue::Engineering(EngineeringQuantityRead { state: EngineeringQuantityState::Value { value_si, origin: BindingOrigin::Override { basis, .. }, .. }, .. }) if value_si > 1.0 && basis.value.value_si == 0.5)
    );
    let missing = q
        .read_field(
            &case,
            &ProjectField::RatingInput(SeparatorRatingInput::GasDensity),
        )
        .unwrap();
    assert!(matches!(
        missing.value,
        ProjectFieldValue::Engineering(EngineeringQuantityRead {
            state: EngineeringQuantityState::Unspecified,
            ..
        })
    ));
}

#[test]
fn context_rejects_same_id_reopen_and_equipment_only_edit_while_reference_survives_rename() {
    let mut app = app();
    let q = ProjectQuery::new(&app.workspace);
    let context = q.input_context();
    let reference = q.reference(ProjectObjectId::Equipment(member().task_id));
    assert!(ProjectQuery::with_context(&app.workspace, &context).is_ok());
    let mut reopened = app.clone();
    reopened.workspace.reset_flow_run_session();
    assert_eq!(
        context.validate(&reopened.workspace),
        Err(ProjectQueryError::InputContextChanged)
    );
    let revision = app.workspace.document.revision;
    app.workspace
        .engineering
        .apply(vec![EngineeringCommand::UpdateRatingLabels {
            task: member().task_id,
            revision: 5,
            tag: "V-NEW".into(),
            name: "Renamed".into(),
        }])
        .unwrap();
    assert_eq!(app.workspace.document.revision, revision);
    assert_eq!(
        context.validate(&app.workspace),
        Err(ProjectQueryError::InputContextChanged)
    );
    assert_eq!(
        ProjectQuery::new(&app.workspace)
            .object(&reference)
            .unwrap()
            .label,
        "V-NEW · Renamed"
    );
    app.workspace.engineering.undo().unwrap();
    assert_eq!(
        context.validate(&app.workspace),
        Err(ProjectQueryError::InputContextChanged)
    );
}

#[test]
fn action_discovery_reports_adapter_semantics_without_executing() {
    let app = app();
    let before = app.clone();
    let q = ProjectQuery::new(&app.workspace);
    let case = q.reference(ProjectObjectId::Case(member()));
    for mode in [ActionExecution::Inline, ActionExecution::Background] {
        let actions = q.actions(Some(&case), mode).unwrap();
        assert!(actions.iter().any(|a| matches!(a, ProjectActionDescriptor::Equipment(d) if matches!(d.kind, EquipmentActionKind::Rate(_)) && d.execution == mode && !d.modifies_inputs && !d.undoable)));
        let actions = q.actions(None, mode).unwrap();
        assert!(actions.iter().any(|a| matches!(a, ProjectActionDescriptor::Flow(d) if d.asynchronous == (mode == ActionExecution::Background))));
    }
    assert_eq!(app, before);
}

#[test]
fn input_context_detects_equal_revision_flow_forks_but_ignores_run_cancellation() {
    let mut app = app();
    let before = ProjectQuery::new(&app.workspace).input_context();
    app.workspace.revoke_flow_run_acceptance();
    assert!(before.validate(&app.workspace).is_ok());
    let mut other = app.clone();
    for (app, value) in [(&mut app, 310.0), (&mut other, 320.0)] {
        let mut flow = app.workspace.document.flowsheet.clone();
        flow.streams.get_mut(&"feed".into()).unwrap().temperature_k = value;
        app.commit_document_change(
            crate::DocumentCommand::SetStreamSpecification {
                stream_id: "feed".into(),
                field: "temperature_k".into(),
                value: crate::CommandValue::Number(value),
            },
            flow,
            UNIX_EPOCH,
        );
    }
    assert_eq!(
        app.workspace.document.revision,
        other.workspace.document.revision
    );
    let context = ProjectQuery::new(&app.workspace).input_context();
    assert_eq!(
        context.validate(&other.workspace),
        Err(ProjectQueryError::InputContextChanged)
    );
}
