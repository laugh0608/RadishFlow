//! Stable rejection codes; human diagnostics are never a protocol discriminator.
use super::{issues, protocol::*};
use rf_ui::engineering::{EngineeringError, editor::*};
use serde_json::{Value, json};

pub fn blocked(reason: Value, message: impl Into<String>) -> Failure {
    let mut f = Failure::new(Stage::Step, Code::ActionRejected, message);
    f.status = Status::Blocked;
    f.details = Some(Box::new(reason));
    f
}
pub fn equipment(error: EquipmentActionError) -> Failure {
    let reason = match &error {
        EquipmentActionError::ReadOnly => json!({"reason":"read_only"}),
        EquipmentActionError::DifferentDocument => json!({"reason":"different_document"}),
        EquipmentActionError::DifferentSession => json!({"reason":"different_session"}),
        EquipmentActionError::StaleInput => json!({"reason":"stale_input"}),
        EquipmentActionError::Edit(e) => edit(e),
    };
    blocked(reason, error.to_string())
}
fn edit(e: &EquipmentEditError) -> Value {
    use EquipmentEditError::*;
    let reason = match e {
        Domain(e) => return domain(e),
        Number(_) => "invalid_number",
        Conversion(_) => "invalid_conversion",
        IncompleteNumber => "incomplete_number",
        SelectSource => "select_source",
        SelectBasis => "select_basis",
        OverrideReason => "override_reason",
        AccessDate => "access_date",
        ObservationValue => "observation_value",
        Field(field, inner) => {
            return json!({"reason":"field","field":match field { EquipmentField::Input(v) => json!({"input":super::enums::SizingField::from(*v)}), EquipmentField::Axial(v) => json!({"axial_section":super::enums::AxialSection::from(*v)}) },"details":edit(inner)});
        }
        RatingGeometryField(_, inner) => {
            return json!({"reason":"geometry_field","details":edit(inner)});
        }
        RatingInput(field, inner) => {
            return json!({"reason":"rating_input","field":super::enums::RatingField::from(*field),"details":edit(inner)});
        }
        UnsupportedEditorAction => "unsupported_action",
        MissingEditor => "missing_editor",
        StaleEditor => "stale_editor",
        DraftTypeMismatch => "draft_type_mismatch",
        PendingDrafts => "pending_drafts",
        MissingTask => "missing_task",
        CounterExhausted => "counter_exhausted",
    };
    json!({"reason":reason})
}
fn domain(e: &EngineeringError) -> Value {
    use EngineeringError::*;
    let reason = match e {
        InvalidBasis(e) => return json!({"reason":"invalid_basis","details":issues::data(e)}),
        InvalidIdentity(e) => {
            return json!({"reason":"invalid_identity","details":issues::identity(e.kind)});
        }
        InvalidBinding { error, .. } => {
            return json!({"reason":"invalid_binding","details":issues::data(error)});
        }
        MissingApplicabilitySource { task, source } => {
            return json!({"reason":"missing_applicability_source","task":task.as_str(),"source":source.as_str()});
        }
        Referenced { .. } => "referenced",
        IdentityUnavailable(_) => "identity_unavailable",
        MissingRecord(_) => "missing_record",
        RevisionConflict {
            expected, supplied, ..
        } => return json!({"reason":"revision_conflict","expected":expected,"supplied":supplied}),
        CaseIdentityChanged(_) => "case_identity_changed",
        UnsupportedRatingMethod(_) => "unsupported_rating_method",
        UnsupportedRatingCaseCount { actual, .. } => {
            return json!({"reason":"unsupported_rating_case_count","actual":actual});
        }
        InvalidCaseOrder(_) => "invalid_case_order",
        InvalidAdoption(e) => {
            return json!({"reason":"invalid_adoption","details":match e {
                rf_model::equipment::GeometryAdoptionError::MissingDocument => "missing_document",
                rf_model::equipment::GeometryAdoptionError::InvalidSourceIdentity => "invalid_source_identity",
                rf_model::equipment::GeometryAdoptionError::InvalidSourceBasis => "invalid_source_basis",
                rf_model::equipment::GeometryAdoptionError::InvalidSourceBinding => "invalid_source_binding",
                rf_model::equipment::GeometryAdoptionError::MissingSourceEvidence => "missing_source_evidence",
                rf_model::equipment::GeometryAdoptionError::InvalidDimension => "invalid_dimension",
                rf_model::equipment::GeometryAdoptionError::MissingAdjustmentReason => "missing_adjustment_reason",
                rf_model::equipment::GeometryAdoptionError::InvalidAxialSection => "invalid_axial_section",
            }});
        }
        CounterExhausted => "counter_exhausted",
        StalePreparation => "stale_preparation",
        StaleCalculation => "stale_calculation",
    };
    json!({"reason":reason})
}
pub fn flow(e: crate::flow_actions::FlowActionError) -> Failure {
    use crate::flow_actions::FlowActionError::*;
    let reason = match &e {
        ReadOnly => "read_only",
        Context(_) => "input_context_changed",
        PendingDrafts => "pending_drafts",
        PendingCanvasEdit => "pending_canvas_edit",
        PortNotBound(_) => "port_not_bound",
        Rejected(e) => e.code().as_str(),
        Connection(e) => return modeling(e),
        Write(e) => match e {
            rf_ui::variable_commands::VariableWriteError::Lookup(_) => "field_missing",
            rf_ui::variable_commands::VariableWriteError::ReadOnly => "read_only_field",
            rf_ui::variable_commands::VariableWriteError::RevisionConflict { .. } => {
                "revision_conflict"
            }
            rf_ui::variable_commands::VariableWriteError::PendingDrafts => "pending_drafts",
            rf_ui::variable_commands::VariableWriteError::TypeMismatch { .. } => "type_mismatch",
            rf_ui::variable_commands::VariableWriteError::Rejected(e) => e.code().as_str(),
        },
    };
    blocked(json!({"reason":reason}), e.to_string())
}
pub fn modeling(e: &crate::modeling_actions::ModelingActionError) -> Failure {
    use crate::modeling_actions::ModelingActionError::*;
    let reason = match e {
        DifferentDocument => "different_document",
        RevisionConflict { .. } => "revision_conflict",
        PendingDrafts => "pending_drafts",
        PendingCanvasEdit => "pending_canvas_edit",
        ConnectionUnavailable => "connection_unavailable",
        AmbiguousConnection => "ambiguous_connection",
        Rejected(e) => e.code().as_str(),
    };
    blocked(json!({"reason":reason}), e.to_string())
}
