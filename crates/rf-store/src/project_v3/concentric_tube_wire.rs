//! Shared v3/v4 formal inputs for the independent concentric-tube rating.

use super::task_wire::MethodWire;
use rf_model::equipment::*;
use rf_types::{EngineeringCaseId, EquipmentTaskId, SourceId};
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
    #[serde(deserialize_with = "object")]
    inlet: InletWire,
    #[serde(deserialize_with = "object")]
    properties: PropertiesWire,
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

impl From<RatingWire> for ConcentricTubeRatingTask {
    fn from(wire: RatingWire) -> Self {
        let case = wire.case;
        let side = |side: SideWire| ConcentricTubeSideInputs {
            inlet: side.inlet.into(),
            properties: side.properties.into(),
        };
        Self {
            id: wire.id,
            revision: wire.revision,
            tag: wire.tag,
            name: wire.name,
            method: EquipmentMethod {
                id: wire.method.id,
                version: wire.method.version,
            },
            geometry: wire.geometry.into(),
            case: ConcentricTubeRatingCase {
                id: case.id,
                revision: case.revision,
                name: case.name,
                purpose: case.purpose,
                hot: side(case.hot),
                cold: side(case.cold),
                limits: case.limits.into(),
            },
        }
    }
}

impl From<&ConcentricTubeRatingTask> for RatingWire {
    fn from(task: &ConcentricTubeRatingTask) -> Self {
        let case = &task.case;
        let side = |side: &ConcentricTubeSideInputs| SideWire {
            inlet: (&side.inlet).into(),
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
