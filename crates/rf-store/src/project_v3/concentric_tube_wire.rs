//! Formal concentric-tube inputs. Stream associations require the v4 envelope.

use super::task_wire::MethodWire;
use rf_model::equipment::*;
use rf_types::{
    EngineeringCaseId, EquipmentTaskId, ObjectCreationId, RfError, RfResult, SourceId, StreamId,
};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{MapAccess, Visitor, value::MapAccessDeserializer},
};
use std::{fmt, marker::PhantomData};

pub(super) const TASK_KIND: &str = "concentricTubeHeatExchangerRating";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct RatingWire {
    id: EquipmentTaskId,
    revision: u64,
    tag: String,
    name: String,
    #[serde(deserialize_with = "object")]
    method: MethodWire,
    #[serde(deserialize_with = "object")]
    geometry: GeometryWire,
    #[serde(deserialize_with = "object")]
    case: CaseWire,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CaseWire {
    id: EngineeringCaseId,
    revision: u64,
    name: String,
    purpose: String,
    #[serde(
        default,
        deserialize_with = "present_object",
        skip_serializing_if = "Option::is_none"
    )]
    flow_association: Option<AssociationWire>,
    #[serde(deserialize_with = "object")]
    hot: SideWire,
    #[serde(deserialize_with = "object")]
    cold: SideWire,
    #[serde(deserialize_with = "object")]
    limits: LimitsWire,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SideWire {
    #[serde(
        default,
        deserialize_with = "present_object",
        skip_serializing_if = "Option::is_none"
    )]
    inlet: Option<InletWire>,
    #[serde(deserialize_with = "object")]
    properties: PropertiesWire,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AssociationWire {
    #[serde(deserialize_with = "object")]
    hot: StreamWire,
    #[serde(deserialize_with = "object")]
    cold: StreamWire,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StreamWire {
    stream_id: StreamId,
    creation_id: String,
}

impl StreamWire {
    fn decode(self, path: &str) -> RfResult<ConcentricTubeFlowStream> {
        Ok(ConcentricTubeFlowStream {
            id: self.stream_id,
            creation_id: self
                .creation_id
                .parse::<ObjectCreationId>()
                .map_err(|error| RfError::invalid_input(format!("{path}.creationId: {error}")))?,
        })
    }
}

impl From<&ConcentricTubeFlowStream> for StreamWire {
    fn from(stream: &ConcentricTubeFlowStream) -> Self {
        Self {
            stream_id: stream.id.clone(),
            creation_id: stream.creation_id.to_string(),
        }
    }
}

fn object<'de, D: Deserializer<'de>, T: Deserialize<'de>>(de: D) -> Result<T, D::Error> {
    struct Object<T>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>> Visitor<'de> for Object<T> {
        type Value = T;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a JSON object")
        }

        fn visit_map<M: MapAccess<'de>>(self, map: M) -> Result<T, M::Error> {
            T::deserialize(MapAccessDeserializer::new(map))
        }
    }
    de.deserialize_map(Object(PhantomData))
}

fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(de: D) -> Result<Option<T>, D::Error> {
    T::deserialize(de).map(Some)
}

fn present_object<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    de: D,
) -> Result<Option<T>, D::Error> {
    object(de).map(Some)
}

// Each listed SI number is optional, rejects explicit null, and is checked
// before writing because serde_json would otherwise turn NaN/inf into null.
macro_rules! si_wire {
    ($wire:ident for $model:ident {
        required { $($required:ident: $ty:ty),* }
        $(optional { $optional:ident: $optional_ty:ty })?
        $(nested { $nested:ident: $nested_ty:ty })?
        numbers { $($number:ident),+ $(,)? }
    }) => {
        #[derive(Serialize, Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct $wire {
            $($required: $ty,)*
            $(
                #[serde(deserialize_with = "object")]
                $nested: $nested_ty,
            )?
            $(
                #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
                $optional: Option<$optional_ty>,
            )?
            $(
                #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
                $number: Option<f64>,
            )+
        }

        impl From<$wire> for $model {
            fn from(wire: $wire) -> Self {
                Self {
                    $($required: wire.$required,)*
                    $($nested: wire.$nested.into(),)?
                    $($optional: wire.$optional,)?
                    $($number: wire.$number,)+
                }
            }
        }

        impl From<&$model> for $wire {
            fn from(model: &$model) -> Self {
                Self {
                    $($required: model.$required.to_owned(),)*
                    $($nested: (&model.$nested).into(),)?
                    $($optional: model.$optional.clone(),)?
                    $($number: model.$number,)+
                }
            }
        }

        impl $wire {
            fn non_finite(model: &$model) -> Option<&'static str> {
                [
                    $((stringify!($number), model.$number)),+
                ].into_iter().find_map(|(field, value)| value.is_some_and(|v| !v.is_finite()).then_some(field))
            }
        }
    };
}

si_wire!(GeometryWire for ConcentricTubeGivenGeometry {
    required { revision: u64 }
    optional { source_id: SourceId }
    numbers {
        active_length_m, inner_tube_inner_diameter_m, inner_tube_outer_diameter_m,
        outer_tube_inner_diameter_m, wall_thermal_conductivity_w_per_m_k,
    }
});

si_wire!(InletWire for ConcentricTubeLiquidInlet {
    required {}
    numbers { temperature_k, pressure_pa, molar_flow_mol_s }
});

si_wire!(PropertiesWire for ConcentricTubeLiquidProperties {
    required { source_id: SourceId }
    nested { liquid_domain: DomainWire }
    numbers {
        molar_mass_kg_per_mol, molar_heat_capacity_j_per_mol_k, density_kg_per_m3,
        dynamic_viscosity_pa_s, thermal_conductivity_w_per_m_k,
    }
});

si_wire!(DomainWire for ConcentricTubeLiquidDomain {
    required {}
    numbers { min_temperature_k, max_temperature_k, min_pressure_pa, max_pressure_pa }
});

si_wire!(LimitsWire for ConcentricTubeRatingLimits {
    required {}
    numbers { min_heat_duty_w, max_hot_pressure_drop_pa, max_cold_pressure_drop_pa }
});

impl RatingWire {
    pub(super) fn decode(self) -> RfResult<ConcentricTubeRatingTask> {
        let case = self.case;
        let flow_association = case
            .flow_association
            .map(|wire| -> RfResult<_> {
                Ok(ConcentricTubeFlowAssociation {
                    hot: wire.hot.decode("case.flowAssociation.hot")?,
                    cold: wire.cold.decode("case.flowAssociation.cold")?,
                })
            })
            .transpose()?;
        for (name, side) in [("hot", &case.hot), ("cold", &case.cold)] {
            match (flow_association.is_some(), side.inlet.is_some()) {
                (true, true) => {
                    return Err(RfError::invalid_input(format!(
                        "case.{name}.inlet: a flow-associated case cannot store inlet numbers"
                    )));
                }
                (false, false) => {
                    return Err(RfError::invalid_input(format!(
                        "case.{name}.inlet: an independent case requires an inlet object"
                    )));
                }
                _ => {}
            }
        }
        let side = |side: SideWire| ConcentricTubeSideInputs {
            inlet: side.inlet.map(Into::into).unwrap_or_default(),
            properties: side.properties.into(),
        };
        Ok(ConcentricTubeRatingTask {
            id: self.id,
            revision: self.revision,
            tag: self.tag,
            name: self.name,
            method: EquipmentMethod {
                id: self.method.id,
                version: self.method.version,
            },
            geometry: self.geometry.into(),
            case: ConcentricTubeRatingCase {
                id: case.id,
                revision: case.revision,
                name: case.name,
                purpose: case.purpose,
                flow_association,
                hot: side(case.hot),
                cold: side(case.cold),
                limits: case.limits.into(),
            },
        })
    }
}

impl From<&ConcentricTubeRatingTask> for RatingWire {
    fn from(task: &ConcentricTubeRatingTask) -> Self {
        let case = &task.case;
        let side = |side: &ConcentricTubeSideInputs| SideWire {
            inlet: case
                .flow_association
                .is_none()
                .then(|| (&side.inlet).into()),
            properties: (&side.properties).into(),
        };
        Self {
            id: task.id.clone(),
            revision: task.revision,
            tag: task.tag.clone(),
            name: task.name.clone(),
            method: MethodWire {
                id: task.method.id.clone(),
                version: task.method.version,
            },
            geometry: (&task.geometry).into(),
            case: CaseWire {
                id: case.id.clone(),
                revision: case.revision,
                name: case.name.clone(),
                purpose: case.purpose.clone(),
                flow_association: case.flow_association.as_ref().map(|association| {
                    AssociationWire {
                        hot: (&association.hot).into(),
                        cold: (&association.cold).into(),
                    }
                }),
                hot: side(&case.hot),
                cold: side(&case.cold),
                limits: (&case.limits).into(),
            },
        }
    }
}

pub(super) fn non_finite_field(task: &ConcentricTubeRatingTask) -> Option<String> {
    let hot = &task.case.hot;
    let cold = &task.case.cold;
    [
        ("geometry", GeometryWire::non_finite(&task.geometry)),
        ("case.hot.inlet", InletWire::non_finite(&hot.inlet)),
        (
            "case.hot.properties",
            PropertiesWire::non_finite(&hot.properties),
        ),
        (
            "case.hot.properties.liquidDomain",
            DomainWire::non_finite(&hot.properties.liquid_domain),
        ),
        ("case.cold.inlet", InletWire::non_finite(&cold.inlet)),
        (
            "case.cold.properties",
            PropertiesWire::non_finite(&cold.properties),
        ),
        (
            "case.cold.properties.liquidDomain",
            DomainWire::non_finite(&cold.properties.liquid_domain),
        ),
        ("case.limits", LimitsWire::non_finite(&task.case.limits)),
    ]
    .into_iter()
    .find_map(|(path, field)| field.map(|field| format!("{path}.{field}")))
}
