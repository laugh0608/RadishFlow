//! Resolve protocol references into formal input records, preserving protected metadata.
use super::{protocol::*, records::*, references::Bindings};
use rf_model::{engineering as d, equipment as e};
use std::collections::BTreeMap;

pub fn binding(value: &Binding, refs: &Bindings) -> Result<d::InputBinding, Failure> {
    Ok(match value {
        Binding::Direct { value, source } => d::InputBinding::Direct {
            value: value.clone().into(),
            source_id: refs.resolve(source, Output::Source)?.into(),
        },
        Binding::Basis { basis } => d::InputBinding::BasisRef {
            basis_entry_id: refs.resolve(basis, Output::Basis)?.into(),
        },
        Binding::Override {
            basis,
            value,
            source,
            reason,
        } => d::InputBinding::Override {
            basis_entry_id: refs.resolve(basis, Output::Basis)?.into(),
            value: value.clone().into(),
            source_id: refs.resolve(source, Output::Source)?.into(),
            reason: reason.clone(),
        },
    })
}
fn bindings<K: Copy + Ord, D: From<K> + Ord>(
    values: &BTreeMap<K, Binding>,
    refs: &Bindings,
) -> Result<BTreeMap<D, d::InputBinding>, Failure> {
    values
        .iter()
        .map(|(k, v)| Ok(((*k).into(), binding(v, refs)?)))
        .collect()
}
fn sources(values: &[IdentityRef], refs: &Bindings) -> Result<Vec<rf_types::SourceId>, Failure> {
    values
        .iter()
        .map(|v| Ok(refs.resolve(v, Output::Source)?.into()))
        .collect()
}
pub fn source(
    old: &d::SourceRecord,
    value: &SourceData,
    refs: &Bindings,
) -> Result<d::SourceRecord, Failure> {
    Ok(d::SourceRecord {
        id: old.id.clone(),
        revision: old.revision,
        kind: value.kind.into(),
        title: value.title.clone(),
        reference: value.reference.clone(),
        locator: value.locator.clone(),
        publication_version: value.publication_version.clone(),
        accessed_on: value
            .accessed_on
            .as_ref()
            .map(|s| s.parse())
            .transpose()
            .map_err(|e: d::EngineeringDataError| {
                let mut failure = Failure::new(Stage::Step, Code::InvalidRequest, e.to_string());
                failure.details = Some(Box::new(super::issues::data(&e)));
                failure
            })?,
        adoption_note: value.adoption_note.clone(),
        review: value.review.into(),
        applicability: d::SourceApplicability {
            fluid_system: value.applicability.fluid_system.clone(),
            phase: value.applicability.phase.into(),
            operating_conditions: value.applicability.operating_conditions.as_ref().map(|v| {
                d::SourceOperatingConditions {
                    temperature_k: v.temperature_k,
                    pressure_pa: v.pressure_pa,
                }
            }),
            scope: value.applicability.scope.clone(),
        },
        observations: value
            .observations
            .iter()
            .map(|o| {
                Ok(d::SourceObservation {
                    id: o.id.clone().into(),
                    quantity: o.quantity.into(),
                    original_value: o.original_value,
                    original_unit: o.original_unit.clone(),
                    canonical_value: o.canonical_value,
                    conversion_note: o.conversion_note.clone(),
                    depends_on: o
                        .depends_on
                        .iter()
                        .map(|r| {
                            Ok(d::SourceObservationRef {
                                source_id: refs.resolve(&r.source, Output::Source)?.into(),
                                observation_id: r.observation_id.clone().into(),
                            })
                        })
                        .collect::<Result<_, Failure>>()?,
                })
            })
            .collect::<Result<_, Failure>>()?,
    })
}
pub fn basis(
    old: &d::BasisEntry,
    value: &BasisData,
    refs: &Bindings,
) -> Result<d::BasisEntry, Failure> {
    Ok(d::BasisEntry {
        id: old.id.clone(),
        revision: old.revision,
        name: value.name.clone(),
        value: value.value.clone().into(),
        source_id: refs.resolve(&value.source, Output::Source)?.into(),
    })
}
pub fn sizing(
    old: &e::SeparatorSizingTask,
    value: &SizingData,
    refs: &Bindings,
) -> Result<e::SeparatorSizingTask, Failure> {
    let mut task = old.clone();
    task.tag = value.tag.clone();
    task.name = value.name.clone();
    task.case.name = value.case.name.clone();
    task.case.purpose = value.case.purpose.clone();
    task.case.fluid_system = value.case.fluid_system.clone();
    let g = &value.case.geometry_applicability;
    let c = &value.case.case_applicability;
    task.case.applicability.orientation = g.orientation.map(Into::into);
    task.case.applicability.gas_direction = g.gas_direction.map(Into::into);
    task.case.applicability.geometry = g.geometry.map(Into::into);
    task.case.applicability.internals_description = g.internals_description.clone();
    task.case.applicability.service = c.service.map(Into::into);
    task.case.applicability.k_basis = c.k_basis.map(Into::into);
    task.case.applicability.flow_basis = c.flow_basis.map(Into::into);
    task.case.applicability_source_ids = sources(&value.case.applicability_sources, refs)?;
    task.case.inputs = bindings(&value.case.inputs, refs)?;
    task.axial_sections = bindings(&value.axial_sections, refs)?;
    Ok(task)
}
pub fn geometry(
    old: &e::SeparatorGivenGeometry,
    value: &GeometryData,
    refs: &Bindings,
) -> Result<e::SeparatorGivenGeometry, Failure> {
    let mut geometry = old.clone();
    geometry.applicability = value.applicability.clone().into();
    geometry.applicability_source_ids = sources(&value.applicability_sources, refs)?;
    geometry.inputs = bindings(&value.inputs, refs)?;
    geometry.axial_sections = bindings(&value.axial_sections, refs)?;
    Ok(geometry)
}
pub fn rating_case(
    old: &e::SeparatorRatingCase,
    value: &RatingCaseData,
    refs: &Bindings,
) -> Result<e::SeparatorRatingCase, Failure> {
    let mut case = old.clone();
    case.name = value.name.clone();
    case.purpose = value.purpose.clone();
    case.fluid_system = value.fluid_system.clone();
    case.applicability = value.applicability.clone().into();
    case.applicability_source_ids = sources(&value.applicability_sources, refs)?;
    case.inputs = bindings(&value.inputs, refs)?;
    Ok(case)
}
