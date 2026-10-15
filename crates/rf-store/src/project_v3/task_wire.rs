//! Strict equipment DTOs. Method capability is probed before decoding this payload.
use super::engineering_wire::{indexed, quantity_id};
use rf_model::{
    engineering::{EngineeringValue, InputBinding},
    equipment::*,
};
use rf_types::{BasisEntryId, EngineeringCaseId, EquipmentTaskId, SourceId, units::QuantityKind};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{MapAccess, Visitor},
};
use std::{collections::BTreeMap, fmt, marker::PhantomData};

pub(super) const TASK_KIND: &str = "verticalGasLiquidSeparatorSizing";
pub(super) const METHOD_ID: &str = "radishflow.separator.vertical-sb-working-volume";
pub(super) const METHOD_VERSION: u32 = 1;

macro_rules! wire_enum {
    ($wire:ident, $model:ident, {$($variant:ident => $name:literal),+ $(,)?}) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        pub(super) enum $wire { $(#[serde(rename = $name)] $variant),+ }
        impl From<$model> for $wire { fn from(value: $model) -> Self { match value { $($model::$variant => Self::$variant),+ } } }
        impl From<$wire> for $model { fn from(value: $wire) -> Self { match value { $($wire::$variant => Self::$variant),+ } } }
    };
}
wire_enum!(InputKey, SeparatorInput, {
    OperatingTemperature => "operatingTemperature", OperatingPressure => "operatingPressure",
    GasVolumeFlow => "gasActualVolumeFlow", LiquidVolumeFlow => "liquidActualVolumeFlow",
    GasDensity => "gasMassDensity", LiquidDensity => "liquidMassDensity", EffectiveK => "effectiveK",
    GasVelocityUtilization => "gasUtilization", WorkingVolumeTime => "workingDuration",
    MaximumWorkingLiquidHeight => "maximumWorkingLiquidHeight"
});
wire_enum!(AxialKey, SeparatorAxialSection, {
    LowerTangentToLowLiquidLevel => "bottomTangentToLLL", NormalToHighLiquidLevel => "nllToHLL",
    HighLiquidLevelToInlet => "hllToInletCenter", InletToInternals => "inletCenterToInternalsBottom",
    InternalsThickness => "internalsThickness", InternalsToUpperTangent => "internalsTopToTopTangent"
});
wire_enum!(Orientation, SeparatorOrientation, {Vertical => "vertical", Horizontal => "horizontal"});
wire_enum!(GasDirection, SeparatorGasDirection, {Upward => "upward", Other => "other"});
wire_enum!(Service, SeparatorService, {CleanNonFoamingTwoPhase => "cleanNonFoamingTwoPhase", SinglePhase => "singlePhase", Foaming => "foaming", Solids => "solids", Slugging => "slugging", ThreePhase => "threePhase"});
wire_enum!(Geometry, SeparatorSectionGeometry, {FullEqualDiameterCylinder => "fullEqualDiameterCylinder", ReducedEffectiveArea => "reducedEffectiveArea", NonCylindrical => "nonCylindrical"});
wire_enum!(KBasis, SeparatorKBasis, {EffectiveWholeVesselForDeclaredInternals => "effectiveWholeVesselForDeclaredInternals", MistEliminatorOnly => "mistEliminatorOnly", UnmatchedInternals => "unmatchedInternals"});
wire_enum!(FlowBasis, SeparatorFlowBasis, {ActualAtOperatingConditions => "actualAtOperatingConditions", StandardConditions => "standardConditions"});

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct TaskWire {
    id: EquipmentTaskId,
    revision: u64,
    tag: String,
    name: String,
    kind: String,
    method: MethodWire,
    case: CaseWire,
    #[serde(deserialize_with = "bindings")]
    inputs: BTreeMap<InputKey, BindingWire>,
    #[serde(
        default,
        deserialize_with = "bindings",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    axial_sections: BTreeMap<AxialKey, BindingWire>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MethodWire {
    id: String,
    version: u32,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CaseWire {
    id: EngineeringCaseId,
    name: String,
    purpose: String,
    fluid_system: String,
    applicability_source_ids: Vec<SourceId>,
    applicability: ApplicabilityWire,
}
// All declaration fields are required. Null explicitly means not yet confirmed;
// enum values retain unsupported declarations instead of collapsing them to true/false.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ApplicabilityWire {
    #[serde(deserialize_with = "declaration")]
    orientation: Option<Orientation>,
    #[serde(deserialize_with = "declaration")]
    gas_direction: Option<GasDirection>,
    #[serde(deserialize_with = "declaration")]
    service: Option<Service>,
    #[serde(deserialize_with = "declaration")]
    geometry: Option<Geometry>,
    #[serde(deserialize_with = "declaration")]
    k_basis: Option<KBasis>,
    #[serde(deserialize_with = "declaration")]
    flow_basis: Option<FlowBasis>,
    internals_description: String,
}
fn declaration<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    de: D,
) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(de)
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum BindingWire {
    #[serde(rename_all = "camelCase")]
    Direct {
        #[serde(with = "quantity_id")]
        quantity_id: QuantityKind,
        value_si: f64,
        source_id: SourceId,
    },
    #[serde(rename_all = "camelCase")]
    BasisRef { basis_entry_id: BasisEntryId },
    #[serde(rename_all = "camelCase")]
    Override {
        basis_entry_id: BasisEntryId,
        #[serde(with = "quantity_id")]
        quantity_id: QuantityKind,
        value_si: f64,
        source_id: SourceId,
        reason: String,
    },
}
impl From<BindingWire> for InputBinding {
    fn from(value: BindingWire) -> Self {
        match value {
            BindingWire::Direct {
                quantity_id,
                value_si,
                source_id,
            } => Self::Direct {
                value: EngineeringValue {
                    quantity: quantity_id,
                    value_si,
                },
                source_id,
            },
            BindingWire::BasisRef { basis_entry_id } => Self::BasisRef { basis_entry_id },
            BindingWire::Override {
                basis_entry_id,
                quantity_id,
                value_si,
                source_id,
                reason,
            } => Self::Override {
                basis_entry_id,
                value: EngineeringValue {
                    quantity: quantity_id,
                    value_si,
                },
                source_id,
                reason,
            },
        }
    }
}
impl From<&InputBinding> for BindingWire {
    fn from(value: &InputBinding) -> Self {
        match value {
            InputBinding::Direct { value, source_id } => Self::Direct {
                quantity_id: value.quantity,
                value_si: value.value_si,
                source_id: source_id.clone(),
            },
            InputBinding::BasisRef { basis_entry_id } => Self::BasisRef {
                basis_entry_id: basis_entry_id.clone(),
            },
            InputBinding::Override {
                basis_entry_id,
                value,
                source_id,
                reason,
            } => Self::Override {
                basis_entry_id: basis_entry_id.clone(),
                quantity_id: value.quantity,
                value_si: value.value_si,
                source_id: source_id.clone(),
                reason: reason.clone(),
            },
        }
    }
}
impl From<TaskWire> for SeparatorSizingTask {
    fn from(t: TaskWire) -> Self {
        let a = t.case.applicability;
        Self {
            id: t.id,
            revision: t.revision,
            tag: t.tag,
            name: t.name,
            method: EquipmentMethod {
                id: t.method.id,
                version: t.method.version,
            },
            case: SeparatorCase {
                id: t.case.id,
                name: t.case.name,
                purpose: t.case.purpose,
                fluid_system: t.case.fluid_system,
                applicability_source_ids: t.case.applicability_source_ids,
                inputs: t
                    .inputs
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect(),
                applicability: SeparatorApplicability {
                    orientation: a.orientation.map(Into::into),
                    gas_direction: a.gas_direction.map(Into::into),
                    service: a.service.map(Into::into),
                    geometry: a.geometry.map(Into::into),
                    k_basis: a.k_basis.map(Into::into),
                    flow_basis: a.flow_basis.map(Into::into),
                    internals_description: a.internals_description,
                },
            },
            axial_sections: t
                .axial_sections
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }
}
impl From<&SeparatorSizingTask> for TaskWire {
    fn from(t: &SeparatorSizingTask) -> Self {
        let a = &t.case.applicability;
        Self {
            id: t.id.clone(),
            revision: t.revision,
            tag: t.tag.clone(),
            name: t.name.clone(),
            kind: TASK_KIND.into(),
            method: MethodWire {
                id: t.method.id.clone(),
                version: t.method.version,
            },
            case: CaseWire {
                id: t.case.id.clone(),
                name: t.case.name.clone(),
                purpose: t.case.purpose.clone(),
                fluid_system: t.case.fluid_system.clone(),
                applicability_source_ids: t.case.applicability_source_ids.clone(),
                applicability: ApplicabilityWire {
                    orientation: a.orientation.map(Into::into),
                    gas_direction: a.gas_direction.map(Into::into),
                    service: a.service.map(Into::into),
                    geometry: a.geometry.map(Into::into),
                    k_basis: a.k_basis.map(Into::into),
                    flow_basis: a.flow_basis.map(Into::into),
                    internals_description: a.internals_description.clone(),
                },
            },
            inputs: t
                .case
                .inputs
                .iter()
                .map(|(k, v)| ((*k).into(), v.into()))
                .collect(),
            axial_sections: t
                .axial_sections
                .iter()
                .map(|(k, v)| ((*k).into(), v.into()))
                .collect(),
        }
    }
}
fn bindings<'de, D: Deserializer<'de>, K: Deserialize<'de> + Ord + fmt::Debug>(
    de: D,
) -> Result<BTreeMap<K, BindingWire>, D::Error> {
    struct Bindings<K>(PhantomData<K>);
    impl<'de, K: Deserialize<'de> + Ord + fmt::Debug> Visitor<'de> for Bindings<K> {
        type Value = BTreeMap<K, BindingWire>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("unique equipment binding fields")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut bindings = BTreeMap::new();
            while let Some(key) = map.next_key::<K>()? {
                if bindings.contains_key(&key) {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate binding {key:?}"
                    )));
                }
                let value = map
                    .next_value()
                    .map_err(|e| serde::de::Error::custom(format!("{key:?}: {e}")))?;
                bindings.insert(key, value);
            }
            Ok(bindings)
        }
    }
    de.deserialize_map(Bindings(PhantomData))
}
// Second pass on ORIGINAL text, after capability probes, retains duplicate checks.
#[derive(Deserialize)]
struct TaskProject {
    engineering: Option<TaskEngineering>,
}
#[derive(Deserialize)]
struct TaskEngineering {
    #[serde(deserialize_with = "tasks")]
    tasks: Vec<TaskWire>,
}
fn tasks<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<TaskWire>, D::Error> {
    indexed(de, "engineering.tasks")
}
pub(super) fn decode(contents: &str) -> rf_types::RfResult<Vec<SeparatorSizingTask>> {
    let project: TaskProject = serde_json::from_str(contents).map_err(super::json_error)?;
    Ok(project
        .engineering
        .map_or_else(Vec::new, |e| e.tasks.into_iter().map(Into::into).collect()))
}

// Only inspect legacy payload shape after every method envelope is supported.
// Arbitrary case payloads for unknown methods remain fully opaque.
#[derive(Deserialize)]
struct LegacyProject {
    engineering: Option<LegacyEngineering>,
}
#[derive(Deserialize)]
struct LegacyEngineering {
    tasks: Vec<LegacyTask>,
}
#[derive(Deserialize)]
struct LegacyTask {
    case: Option<LegacyCase>,
}
#[derive(Deserialize)]
struct LegacyCase {
    configuration: Option<serde::de::IgnoredAny>,
}
pub(super) fn legacy_payload(contents: &str) -> rf_types::RfResult<bool> {
    let project: LegacyProject = serde_json::from_str(contents).map_err(super::json_error)?;
    Ok(project.engineering.is_some_and(|e| {
        e.tasks
            .iter()
            .any(|t| t.case.as_ref().is_some_and(|c| c.configuration.is_some()))
    }))
}
