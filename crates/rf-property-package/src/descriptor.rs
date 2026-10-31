use rf_thermo::{ThermoMethod, ThermoSystem};
use rf_types::ComponentId;

use crate::PropertyPackageManifest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermoCapability {
    KValues,
    BubbleDewPressures,
    BubbleDewTemperatures,
    TpFlash,
    LiquidSensibleEnthalpy,
    VaporSensibleEnthalpy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentDataKind {
    Antoine,
    LiquidHeatCapacity,
    VaporHeatCapacity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentDescriptor {
    pub id: ComponentId,
    pub name: String,
    pub missing_data: Vec<ComponentDataKind>,
}

/// Implementation support is separate from data coverage and physical accuracy.
/// The current package schema has no formula, validity range or accuracy evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyPackageDescriptor {
    pub manifest: PropertyPackageManifest,
    pub method: ThermoMethod,
    pub components: Vec<ComponentDescriptor>,
    pub implemented_capabilities: Vec<ThermoCapability>,
}

impl PropertyPackageDescriptor {
    pub(crate) fn new(manifest: PropertyPackageManifest, system: &ThermoSystem) -> Self {
        let components = system
            .components
            .iter()
            .map(|component| {
                let mut missing_data = Vec::new();
                if component.antoine.is_none() {
                    missing_data.push(ComponentDataKind::Antoine);
                }
                if component.liquid_heat_capacity_j_per_mol_k.is_none() {
                    missing_data.push(ComponentDataKind::LiquidHeatCapacity);
                }
                if component.vapor_heat_capacity_j_per_mol_k.is_none() {
                    missing_data.push(ComponentDataKind::VaporHeatCapacity);
                }
                ComponentDescriptor {
                    id: component.id.clone(),
                    name: component.name.clone(),
                    missing_data,
                }
            })
            .collect();
        Self {
            manifest,
            method: system.method,
            components,
            implemented_capabilities: vec![
                ThermoCapability::KValues,
                ThermoCapability::BubbleDewPressures,
                ThermoCapability::BubbleDewTemperatures,
                ThermoCapability::TpFlash,
                ThermoCapability::LiquidSensibleEnthalpy,
                ThermoCapability::VaporSensibleEnthalpy,
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageDataOrigin {
    InMemory,
    Cache,
    LocalFiles,
    Bundled,
}

/// A content fingerprint identifies the validated canonical payload, not its publisher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageProvenance {
    pub origin: PackageDataOrigin,
    pub manifest_schema_version: u32,
    pub payload_schema_version: u32,
    pub payload_fingerprint: String,
    pub canonical_payload_size_bytes: u64,
    pub thermo_implementation: &'static str,
    pub flash_implementation: &'static str,
}
