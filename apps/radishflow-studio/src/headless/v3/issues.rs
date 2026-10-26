//! Structured diagnostic projections retain domain targets and reason variants.
use super::{enums::*, protocol::IdentityRef};
use rf_model::{
    engineering::{EngineeringDataError, EngineeringDataErrorKind},
    equipment::EquipmentIdentityErrorKind,
};
use rf_ui::engineering::editor::{separator_rating as rating, separator_sizing as sizing};
use serde_json::{Value, json};

pub fn data(error: &EngineeringDataError) -> Value {
    use EngineeringDataErrorKind::*;
    let reason = match &error.kind {
        EmptyId => json!({"code":"empty_id"}),
        DuplicateId => json!({"code":"duplicate_id"}),
        NonFiniteValue => json!({"code":"non_finite_value"}),
        InvalidOperatingConditions => json!({"code":"invalid_operating_conditions"}),
        InvalidAccessDate => json!({"code":"invalid_access_date"}),
        MissingSource(id) => json!({"code":"missing_source","source":IdentityRef::id(id)}),
        MissingBasis(id) => json!({"code":"missing_basis","basis":IdentityRef::id(id)}),
        MissingObservation(r) => {
            json!({"code":"missing_observation","source":IdentityRef::id(&r.source_id),"observation_id":r.observation_id.as_str()})
        }
        ObservationCycle => json!({"code":"observation_cycle"}),
        QuantityMismatch { expected, actual } => {
            json!({"code":"quantity_mismatch","expected":Quantity::from(*expected),"actual":Quantity::from(*actual)})
        }
        MissingOverrideReason => json!({"code":"missing_override_reason"}),
    };
    json!({"path":error.path,"reason":reason})
}
pub fn identity(kind: EquipmentIdentityErrorKind) -> &'static str {
    use EquipmentIdentityErrorKind::*;
    match kind {
        EmptyTaskId => "empty_task_id",
        DuplicateTaskId => "duplicate_task_id",
        EmptyCaseId => "empty_case_id",
        EmptyCaseCollection => "empty_case_collection",
        DuplicateCaseId => "duplicate_case_id",
        DuplicateTag => "duplicate_tag",
    }
}
pub fn sizing(issue: &sizing::SizingIssue) -> Value {
    use sizing::{IssueTarget as T, SizingIssueKind as K};
    let target = match &issue.target {
        T::Task => json!({"kind":"task"}),
        T::Case => json!({"kind":"case"}),
        T::Method => json!({"kind":"method"}),
        T::Basis => json!({"kind":"basis"}),
        T::Calculation => json!({"kind":"calculation"}),
        T::Applicability(f) => {
            json!({
                "kind": "applicability",
                "field": match f {
                    sizing::ApplicabilityField::Orientation=>"orientation",
                    sizing::ApplicabilityField::GasDirection=>"gas_direction",
                    sizing::ApplicabilityField::Service=>"service",
                    sizing::ApplicabilityField::Geometry=>"geometry",
                    sizing::ApplicabilityField::KBasis=>"k_basis",
                    sizing::ApplicabilityField::FlowBasis=>"flow_basis",
                    sizing::ApplicabilityField::Internals=>"internals_description"
                }
            })
        }
        T::Input(f) => json!({"kind":"input","field":SizingField::from(*f)}),
        T::AxialSection(f) => json!({"kind":"axial_section","field":AxialSection::from(*f)}),
        T::Source(id) => json!({"kind":"source","id":id.as_str()}),
    };
    let reason = match &issue.kind {
        K::MissingInput => json!({"code":"missing_input"}),
        K::InvalidIdentity(k) => json!({"code":"invalid_identity","reason":identity(*k)}),
        K::InvalidBinding(e) => json!({"code":"invalid_binding","details":data(e)}),
        K::UnsupportedMethod => json!({"code":"unsupported_method"}),
        K::OutsideMethodScope => json!({"code":"outside_method_scope"}),
        K::NonPositiveInput => json!({"code":"non_positive_input"}),
        K::UtilizationAboveOne => json!({"code":"utilization_above_one"}),
        K::AbsentPhase => json!({"code":"absent_phase"}),
        K::InvalidDensityOrder => json!({"code":"invalid_density_order"}),
        K::MissingSourceEvidence => json!({"code":"missing_source_evidence"}),
        K::MissingSource(id) => json!({"code":"missing_source","id":id.as_str()}),
        K::SourceNotAdopted => json!({"code":"source_not_adopted"}),
        K::SourceConditionMismatch => json!({"code":"source_condition_mismatch"}),
        K::NumericRange(step) => {
            json!({"code":"numeric_range","calculation_step":sizing_step(*step)})
        }
    };
    json!({"target":target,"reason":reason})
}
fn sizing_step(step: sizing::CalculationStep) -> &'static str {
    use sizing::CalculationStep::*;
    match step {
        DensityRatio => "density_ratio",
        SoudersBrownVelocity => "souders_brown_velocity",
        AllowedVelocity => "allowed_velocity",
        GasArea => "gas_area",
        WorkingVolume => "working_volume",
        LiquidArea => "liquid_area",
        DiameterSquared => "diameter_squared",
        Diameter => "diameter",
        WorkingHeight => "working_height",
        GasSuperficialVelocity => "gas_superficial_velocity",
        GasLoad => "gas_load",
        TangentHeight => "tangent_height",
        BufferVolume => "buffer_volume",
    }
}
pub fn rating(issue: &rating::RatingIssue) -> Value {
    use rating::{RatingIssueKind as K, RatingIssueTarget as T};
    let target = match &issue.target {
        T::Task => json!({"kind":"task"}),
        T::Method => json!({"kind":"method"}),
        T::Basis => json!({"kind":"basis"}),
        T::Geometry => json!({"kind":"geometry"}),
        T::Calculation => json!({"kind":"calculation"}),
        T::Case(id) => json!({"kind":"case","case_id":id.as_str()}),
        T::Input { case_id, field } => {
            json!({"kind":"input","case_id":case_id.as_str(),"field":RatingField::from(*field)})
        }
        T::GeometryInput(f) => json!({"kind":"geometry_input","field":GeometryField::from(*f)}),
        T::AxialSection(f) => json!({"kind":"axial_section","field":AxialSection::from(*f)}),
        T::Source(id) => json!({"kind":"source","id":id.as_str()}),
        T::GeometryApplicability(f) => {
            json!({
                "kind": "geometry_applicability",
                "field": match f {
                    rating::GeometryApplicabilityField::Orientation=>"orientation",
                    rating::GeometryApplicabilityField::GasDirection=>"gas_direction",
                    rating::GeometryApplicabilityField::Geometry=>"geometry",
                    rating::GeometryApplicabilityField::Internals=>"internals_description"
                }
            })
        }
        T::CaseApplicability { case_id, field } => {
            json!({
                "kind": "case_applicability",
                "case_id": case_id.as_str(),
                "field": match field {
                    rating::CaseApplicabilityField::Service=>"service",
                    rating::CaseApplicabilityField::KBasis=>"k_basis",
                    rating::CaseApplicabilityField::FlowBasis=>"flow_basis"
                }
            })
        }
    };
    let reason = match &issue.kind {
        K::MissingInput => json!({"code":"missing_input"}),
        K::InvalidIdentity(k) => json!({"code":"invalid_identity","reason":identity(*k)}),
        K::InvalidBinding(e) => json!({"code":"invalid_binding","details":data(e)}),
        K::UnsupportedMethod => json!({"code":"unsupported_method"}),
        K::UnsupportedCaseCount { actual } => {
            json!({"code":"unsupported_case_count","actual":actual})
        }
        K::OutsideMethodScope => json!({"code":"outside_method_scope"}),
        K::NonPositiveInput => json!({"code":"non_positive_input"}),
        K::UtilizationAboveOne => json!({"code":"utilization_above_one"}),
        K::AbsentPhase => json!({"code":"absent_phase"}),
        K::InvalidDensityOrder => json!({"code":"invalid_density_order"}),
        K::MissingSourceEvidence => json!({"code":"missing_source_evidence"}),
        K::MissingSource(id) => json!({"code":"missing_source","id":id.as_str()}),
        K::SourceNotAdopted => json!({"code":"source_not_adopted"}),
        K::SourceConditionMismatch => json!({"code":"source_condition_mismatch"}),
        K::NumericRange(step) => {
            json!({"code":"numeric_range","calculation_step":rating_step(*step)})
        }
    };
    json!({"target":target,"reason":reason})
}
fn rating_step(step: rating::CalculationStep) -> &'static str {
    use rating::CalculationStep::*;
    match step {
        DiameterSquared => "diameter_squared",
        GivenArea => "given_area",
        AvailableWorkingVolume => "available_working_volume",
        DensityRatio => "density_ratio",
        SoudersBrownVelocity => "souders_brown_velocity",
        AllowedVelocity => "allowed_velocity",
        GasFlowLimit => "gas_flow_limit",
        RequiredWorkingVolume => "required_working_volume",
        AvailableWorkingTime => "available_working_time",
        LiquidFlowLimit => "liquid_flow_limit",
        GasSuperficialVelocity => "gas_superficial_velocity",
        GasLoad => "gas_load",
        LiquidLoad => "liquid_load",
        TangentHeight => "tangent_height",
        BufferVolume => "buffer_volume",
    }
}
