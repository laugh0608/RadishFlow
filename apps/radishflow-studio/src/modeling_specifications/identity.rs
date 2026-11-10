use super::*;
use rf_types::{DiagnosticPortTarget, ErrorCode};
use rf_ui::variable_browser::{ObjectId, VariableField, VariableId, VariableSection};

/// Navigation destinations are typed and independent of localized labels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecificationTarget {
    Palette,
    ProjectComponents,
    Unit(UnitId),
    Stream(StreamId),
    Field {
        object: ObjectId,
        field: VariableField,
    },
    Composition(StreamId),
    Port(DiagnosticPortTarget),
}

impl SpecificationTarget {
    pub fn exists(&self, document: &FlowsheetDocument) -> bool {
        let flowsheet = &document.flowsheet;
        match self {
            Self::Palette | Self::ProjectComponents => true,
            Self::Unit(id) => flowsheet.units.contains_key(id),
            Self::Stream(id) | Self::Composition(id) => flowsheet.streams.contains_key(id),
            Self::Field { object, .. } => match object {
                ObjectId::Unit(id) => flowsheet.units.contains_key(id),
                ObjectId::Stream(id) => flowsheet.streams.contains_key(id),
            },
            // A missing port signature still needs navigation to its owning unit.
            Self::Port(port) => flowsheet.units.contains_key(&port.unit_id),
        }
    }

    pub fn variable(&self, document: &DocumentId) -> Option<VariableId> {
        match self {
            Self::Field { object, field } => Some(VariableId {
                document: document.clone(),
                object: object.clone(),
                section: VariableSection::Inputs,
                field: field.clone(),
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecificationProblemReason {
    EmptyFlowsheet,
    Topology {
        code: ErrorCode,
        diagnostic_code: Option<String>,
    },
    MissingProjectComponents,
    ComponentOutsideProject(ComponentId),
    InvalidFeedInput,
    MissingFeedComposition,
    InvalidMoleFraction,
    InvalidCompositionTotal,
    UnnormalizedComposition,
    MissingParameter,
    InvalidParameter,
    OutletPressureAboveInlet,
}

/// Values, object names, revision and translated prose are evidence, never keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecificationProblemKey {
    pub document_id: DocumentId,
    pub session: ModelingCheckSessionId,
    pub rule: SpecificationRule,
    pub rule_version: u32,
    pub scope: SpecificationCheckScope,
    pub targets: Vec<SpecificationTarget>,
    pub reason: SpecificationProblemReason,
}

impl SpecificationProblem {
    pub fn scope(&self) -> SpecificationCheckScope {
        use SpecificationCheckScope as S;
        match self {
            Self::EmptyFlowsheet | Self::Topology(_) => S::Topology,
            Self::MissingProjectComponents | Self::ComponentOutsideProject { .. } => {
                S::ProjectComponents
            }
            Self::InvalidFeedInput { .. } | Self::MissingFeedComposition { .. } => S::FeedInputs,
            Self::InvalidMoleFraction { .. }
            | Self::InvalidCompositionTotal { .. }
            | Self::UnnormalizedComposition { .. } => S::StreamCompositions,
            Self::MissingParameter { unit_id, .. } | Self::InvalidParameter { unit_id, .. } => {
                S::UnitParameters(unit_id.clone())
            }
            Self::OutletPressureAboveInlet { unit_id, .. } => S::InletPressure(unit_id.clone()),
        }
    }

    pub fn targets(&self) -> Vec<SpecificationTarget> {
        use SpecificationTarget as T;
        let field = |object, field| T::Field { object, field };
        match self {
            Self::EmptyFlowsheet => vec![T::Palette],
            Self::MissingProjectComponents => vec![T::ProjectComponents],
            Self::Topology(error) => {
                let context = error.context();
                let mut targets: Vec<_> = context
                    .related_port_targets()
                    .iter()
                    .cloned()
                    .map(T::Port)
                    .collect();
                targets.extend(
                    context
                        .related_unit_ids()
                        .iter()
                        .filter(|id| {
                            !context
                                .related_port_targets()
                                .iter()
                                .any(|p| &p.unit_id == *id)
                        })
                        .cloned()
                        .map(T::Unit),
                );
                targets.extend(context.related_stream_ids().iter().cloned().map(T::Stream));
                targets
            }
            Self::ComponentOutsideProject {
                stream_id,
                component_id,
            }
            | Self::InvalidMoleFraction {
                stream_id,
                component_id,
                ..
            } => vec![field(
                ObjectId::Stream(stream_id.clone()),
                VariableField::MoleFraction(component_id.clone()),
            )],
            Self::InvalidFeedInput {
                stream_id,
                field: input,
                ..
            } => vec![field(
                ObjectId::Stream(stream_id.clone()),
                match input {
                    FeedInputField::Temperature => VariableField::Temperature,
                    FeedInputField::Pressure => VariableField::Pressure,
                    FeedInputField::MolarFlow => VariableField::MolarFlow,
                },
            )],
            Self::MissingFeedComposition { stream_id, .. }
            | Self::InvalidCompositionTotal { stream_id, .. }
            | Self::UnnormalizedComposition { stream_id, .. } => {
                vec![T::Composition(stream_id.clone())]
            }
            Self::MissingParameter { unit_id, parameter }
            | Self::InvalidParameter {
                unit_id, parameter, ..
            } => match parameter {
                UnitParameter::OutletASplitFraction => vec![T::Unit(unit_id.clone())],
                UnitParameter::OutletTemperature => vec![field(
                    ObjectId::Unit(unit_id.clone()),
                    VariableField::OutletTemperature,
                )],
                UnitParameter::OutletPressure => vec![field(
                    ObjectId::Unit(unit_id.clone()),
                    VariableField::OutletPressure,
                )],
            },
            Self::OutletPressureAboveInlet { unit_id, .. } => vec![field(
                ObjectId::Unit(unit_id.clone()),
                VariableField::OutletPressure,
            )],
        }
    }

    pub fn key(&self, identity: &SpecificationReportIdentity) -> SpecificationProblemKey {
        use SpecificationProblemReason as R;
        let reason = match self {
            Self::EmptyFlowsheet => R::EmptyFlowsheet,
            Self::Topology(error) => R::Topology {
                code: error.code(),
                diagnostic_code: error.context().diagnostic_code().map(str::to_owned),
            },
            Self::MissingProjectComponents => R::MissingProjectComponents,
            Self::ComponentOutsideProject { component_id, .. } => {
                R::ComponentOutsideProject(component_id.clone())
            }
            Self::InvalidFeedInput { .. } => R::InvalidFeedInput,
            Self::MissingFeedComposition { .. } => R::MissingFeedComposition,
            Self::InvalidMoleFraction { .. } => R::InvalidMoleFraction,
            Self::InvalidCompositionTotal { .. } => R::InvalidCompositionTotal,
            Self::UnnormalizedComposition { .. } => R::UnnormalizedComposition,
            Self::MissingParameter { .. } => R::MissingParameter,
            Self::InvalidParameter { .. } => R::InvalidParameter,
            Self::OutletPressureAboveInlet { .. } => R::OutletPressureAboveInlet,
        };
        SpecificationProblemKey {
            document_id: identity.document_id.clone(),
            session: identity.session,
            rule: self.rule(),
            rule_version: self.rule().version(),
            scope: self.scope(),
            targets: self.targets(),
            reason,
        }
    }
}
