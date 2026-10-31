//! Query only qualified current values; origins remain frozen with their result.
use super::{enums::*, issues, records, references::Executions};
use crate::StudioRunId;
use rf_types::units::QuantityKind;
use rf_ui::{
    engineering::editor::{separator_rating as rating, separator_sizing as sizing},
    engineering::*,
    result_query::*,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub fn measure(value: f64, quantity: QuantityKind) -> Value {
    let q = quantity.definition();
    json!({"value_si":value, "quantity":q.id, "unit":q.canonical_unit.definition().symbol})
}
pub fn flow(app: &rf_ui::AppState, executions: &Executions) -> Value {
    match app.workspace.flow_result() {
        FlowResultView::Current { origin, .. } => {
            let fields = rf_ui::variable_browser::VariableBrowser::from_workspace(&app.workspace)
                .search("")
                .into_iter()
                .filter(|v| v.id.section == rf_ui::variable_browser::VariableSection::Results)
                .map(super::query::flow_field)
                .collect::<Vec<_>>();
            json!({"status":"current", "origin":flow_origin(origin,executions),"fields":fields})
        }
        FlowResultView::Stale { origin, reason } => {
            json!({"status":"stale","origin":origin.map(|v|flow_origin(v,executions)),"reason":flow_reason(reason)})
        }
        FlowResultView::Missing { reason } => {
            json!({"status":"missing","reason":flow_reason(reason)})
        }
        FlowResultView::Failed { origin, diagnostic } => {
            json!({"status":"failed","origin":flow_origin(origin,executions),"diagnostic":flow_diagnostic(diagnostic)})
        }
    }
}
pub fn flow_diagnostic(v: &rf_ui::DiagnosticSummary) -> Value {
    let severity = match v.highest_severity {
        rf_ui::DiagnosticSeverity::Info => "info",
        rf_ui::DiagnosticSeverity::Warning => "warning",
        rf_ui::DiagnosticSeverity::Error => "error",
    };
    json!({
        "document_revision": v.document_revision,
        "severity": severity,
        "code": v.primary_code,
        "message": v.primary_message,
        "count": v.diagnostic_count,
        "unit_ids": v.related_unit_ids.iter().map(|v|v.as_str()).collect::<Vec<_>>(),
        "stream_ids": v.related_stream_ids.iter().map(|v|v.as_str()).collect::<Vec<_>>(),
        "ports": v.related_port_targets.iter().map(|p|json!({
            "unit_id": p.unit_id.as_str(),
            "port": p.port_name
        })).collect::<Vec<_>>()
    })
}
fn flow_reason(v: FlowResultUnavailable) -> &'static str {
    match v {
        FlowResultUnavailable::NoResult => "no_result",
        FlowResultUnavailable::PlanChanged => "plan_changed",
        FlowResultUnavailable::InputChanged => "input_changed",
        FlowResultUnavailable::DifferentSession => "different_session",
        FlowResultUnavailable::UnqualifiedSnapshot => "unqualified_snapshot",
    }
}
fn flow_origin(v: &FlowResultOrigin, executions: &Executions) -> Value {
    json!({
        "document_id": v.document_id().as_str(),
        "revision": v.revision(),
        "execution": v.execution().map(|e|json!({
            "step": executions.step(&StudioRunId::Flow(e.run_id.clone())),
            "scope": "request",
            "solver_implementation": e.solver_implementation,
            "package": e.package.as_ref().map(|p|json!({
                "id": p.package_id,
                "version": p.version,
                "origin": match p.origin {
                    ResultDataOrigin::InMemory=>"in_memory",
                    ResultDataOrigin::Cache=>"cache",
                    ResultDataOrigin::LocalFiles=>"local_files",
                    ResultDataOrigin::Bundled=>"bundled"
                },
                "manifest_schema_version": p.manifest_schema_version,
                "payload_schema_version": p.payload_schema_version,
                "payload_fingerprint": p.payload_fingerprint,
                "canonical_payload_size_bytes": p.canonical_payload_size_bytes,
                "liquid_model": p.liquid_model,
                "vapor_model": p.vapor_model,
                "thermo_implementation": p.thermo_implementation,
                "flash_implementation": p.flash_implementation
            }))
        }))
    })
}
pub fn equipment(view: EquipmentResultView<'_>, executions: &Executions) -> Value {
    match view {
        EquipmentResultView::Current {
            origin,
            value,
            pending,
        } => {
            json!({
                "status": "current",
                "origin": equipment_origin(origin,
                executions),
                "pending": pending,
                "value": match value {
                    EquipmentResultValue::Sizing(v)=>sizing(v),
                    EquipmentResultValue::Rating(v)=>rating(v)
                }
            })
        }
        EquipmentResultView::Stale {
            origin,
            reason,
            pending,
        } => {
            json!({"status":"stale","origin":equipment_origin(origin,executions),"reason":equipment_reason(reason),"pending":pending})
        }
        EquipmentResultView::Missing { reason, pending } => {
            json!({"status":"missing","reason":equipment_reason(reason),"pending":pending})
        }
        EquipmentResultView::Failed {
            origin,
            failure,
            pending,
        } => {
            json!({
                "status": "failed",
                "origin": equipment_origin(origin,
                executions),
                "pending": pending,
                "issues": match failure {
                    EquipmentResultFailure::Sizing(v)=>v.issues.iter().map(issues::sizing).collect::<Vec<_>>(),
                    EquipmentResultFailure::Rating(v)=>v.issues.iter().map(issues::rating).collect::<Vec<_>>()
                }
            })
        }
    }
}
fn equipment_reason(v: EquipmentResultUnavailable) -> &'static str {
    use EquipmentResultUnavailable::*;
    match v {
        NoResult => "no_result",
        MemberMissing => "member_missing",
        InputChanged => "input_changed",
        EvidenceChanged => "evidence_changed",
        InputAndEvidenceChanged => "input_and_evidence_changed",
        EligibilityRevoked => "eligibility_revoked",
    }
}
fn equipment_origin(v: &EquipmentResultOrigin, executions: &Executions) -> Value {
    json!({
        "document_id": v.document_id.as_str(),
        "execution": {
            "scope": "request",
            "step": executions.step(&StudioRunId::Equipment(v.calculation_id.clone()))
        },
        "task_id": v.member.task_id.as_str(),
        "case_id": v.member.case_id.as_str(),
        "method": records::method(&v.method),
        "input_generation": v.generation.input,
        "evidence_generation": v.generation.evidence,
        "task_revision": v.task_revision,
        "case_revision": v.case_revision,
        "geometry_revision": v.geometry_revision,
        "task_tag": v.task_tag,
        "task_name": v.task_name,
        "case_name": v.case_name,
        "basis_revisions": v.basis_revisions.iter().map(|(id,
        r)|(id.as_str(),
        r)).collect::<BTreeMap<_,
        _>>(),
        "source_revisions": v.source_revisions.iter().map(|(id,
        r)|(id.as_str(),
        r)).collect::<BTreeMap<_,
        _>>()
    })
}
pub fn sizing(v: &sizing::SeparatorCandidate) -> Value {
    use QuantityKind::*;
    let d = v.dimensions();
    let c = v.capture();
    json!({
        "kind": "sizing",
        "dimensions": {
            "souders_brown_velocity": measure(d.souders_brown_velocity_m_s,
            Velocity),
            "allowed_gas_velocity": measure(d.allowed_gas_velocity_m_s,
            Velocity),
            "gas_area": measure(d.gas_area_m2,
            Area),
            "liquid_area": measure(d.liquid_area_m2,
            Area),
            "required_area": measure(d.required_area_m2,
            Area),
            "working_volume": measure(d.working_volume_m3,
            Volume),
            "minimum_diameter": measure(d.minimum_diameter_m,
            Length),
            "working_liquid_height": measure(d.working_liquid_height_m,
            Length),
            "gas_load": measure(d.gas_load_fraction,
            DimensionlessRatio),
            "tangent_to_tangent_height": d.tangent_to_tangent_height_m.map(|v|measure(v,
            Length)),
            "buffer_volume": d.buffer_volume_m3.map(|v|measure(v,
            Volume)),
            "controlling_constraint": match d.controlling_constraint {
                sizing::ControllingConstraint::GasArea=>"gas_area",
                sizing::ControllingConstraint::LiquidWorkingVolume=>"liquid_working_volume",
                sizing::ControllingConstraint::Both=>"both"
            }
        },
        "capture": {
            "task": records::sizing(&c.task),
            "values": c.values_si.iter().map(|(k,
            v)|(SizingField::from(*k),
            measure(*v,
            k.quantity()))).collect::<BTreeMap<_,
            _>>(),
            "axial_lengths": c.axial_lengths_m.iter().map(|(k,
            v)|(AxialSection::from(*k),
            measure(*v,
            Length))).collect::<BTreeMap<_,
            _>>(),
            "basis": c.basis_entries.values().map(records::basis).collect::<Vec<_>>(),
            "sources": c.sources.values().map(records::source).collect::<Vec<_>>()
        },
        "assumed_sources": v.assumed_sources().iter().map(|v|v.as_str()).collect::<Vec<_>>(),
        "pending_review_sources": v.pending_review_sources().iter().map(|v|v.as_str()).collect::<Vec<_>>(),
        "missing_axial_sections": v.missing_axial_sections().iter().copied().map(AxialSection::from).collect::<Vec<_>>(),
        "height_issues": v.height_issues().iter().map(issues::sizing).collect::<Vec<_>>(),
        "unchecked_requirements": v.unchecked_requirements().iter().map(unchecked).collect::<Vec<_>>()
    })
}
fn constraint(v: &rating::ConstraintAssessment) -> Value {
    json!({
        "load": measure(v.load_fraction,
        QuantityKind::DimensionlessRatio),
        "margin": measure(v.margin_fraction,
        QuantityKind::DimensionlessRatio),
        "status": constraint_status(v.status)
    })
}
fn constraint_status(v: rating::ConstraintStatus) -> &'static str {
    match v {
        rating::ConstraintStatus::WithinCriteria => "within_criteria",
        rating::ConstraintStatus::BoundaryReview => "boundary_review",
        rating::ConstraintStatus::Exceeded => "exceeded",
    }
}
fn rating(v: &rating::SeparatorRatingResult) -> Value {
    use QuantityKind::*;
    let d = v.performance();
    let c = v.capture();
    json!({
        "kind": "rating",
        "performance": {
            "given_area": measure(d.given_area_m2,
            Area),
            "available_working_volume": measure(d.available_working_volume_m3,
            Volume),
            "required_working_volume": measure(d.required_working_volume_m3,
            Volume),
            "souders_brown_velocity": measure(d.souders_brown_velocity_m_s,
            Velocity),
            "allowed_gas_velocity": measure(d.allowed_gas_velocity_m_s,
            Velocity),
            "actual_gas_velocity": measure(d.actual_gas_velocity_m_s,
            Velocity),
            "gas_flow_limit": measure(d.gas_flow_limit_m3_s,
            ActualVolumeFlow),
            "liquid_flow_limit": measure(d.liquid_flow_limit_m3_s,
            ActualVolumeFlow),
            "available_working_time": measure(d.available_working_time_s,
            Duration),
            "gas": constraint(&d.gas),
            "liquid": constraint(&d.liquid),
            "conclusion": constraint_status(d.conclusion()),
            "tangent_to_tangent_height": d.tangent_to_tangent_height_m.map(|v|measure(v,
            Length)),
            "buffer_volume": d.buffer_volume_m3.map(|v|measure(v,
            Volume))
        },
        "capture": {
            "task_id": c.input.task_id.as_str(),
            "task_revision": c.input.task_revision,
            "tag": c.input.tag,
            "name": c.input.name,
            "method": records::method(&c.input.method),
            "geometry": records::geometry(&c.input.geometry),
            "case": records::rating_case(&c.input.case),
            "geometry_values": c.geometry_values_m.iter().map(|(k,
            v)|(GeometryField::from(*k),
            measure(*v,
            Length))).collect::<BTreeMap<_,
            _>>(),
            "values": c.values_si.iter().map(|(k,
            v)|(RatingField::from(*k),
            measure(*v,
            k.quantity()))).collect::<BTreeMap<_,
            _>>(),
            "axial_lengths": c.axial_lengths_m.iter().map(|(k,
            v)|(AxialSection::from(*k),
            measure(*v,
            Length))).collect::<BTreeMap<_,
            _>>(),
            "basis": c.basis_entries.values().map(records::basis).collect::<Vec<_>>(),
            "sources": c.sources.values().map(records::source).collect::<Vec<_>>()
        },
        "assumed_sources": v.assumed_sources().iter().map(|v|v.as_str()).collect::<Vec<_>>(),
        "pending_review_sources": v.pending_review_sources().iter().map(|v|v.as_str()).collect::<Vec<_>>(),
        "missing_axial_sections": v.missing_axial_sections().iter().copied().map(AxialSection::from).collect::<Vec<_>>(),
        "height_issues": v.height_issues().iter().map(issues::rating).collect::<Vec<_>>(),
        "unchecked_requirements": v.unchecked_requirements().iter().map(unchecked).collect::<Vec<_>>()
    })
}
fn unchecked(v: &sizing::UncheckedRequirement) -> &'static str {
    use sizing::UncheckedRequirement::*;
    match v {
        PhaseEquilibrium => "phase_equilibrium",
        EntrainmentAndEfficiency => "entrainment_and_efficiency",
        PressureDrop => "pressure_drop",
        SlugHandling => "slug_handling",
        MechanicalDesignAndCertification => "mechanical_design_and_certification",
        StandardDiameterRounding => "standard_diameter_rounding",
        MultipleOperatingCases => "multiple_operating_cases",
    }
}
