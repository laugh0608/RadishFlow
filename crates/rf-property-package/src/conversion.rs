use crate::{PropertyPackageClassification, PropertyPackageManifest, PropertyPackageSource};
use rf_store::{
    StoredAntoineCoefficients, StoredLiquidPhaseModel, StoredPropertyPackageClassification,
    StoredPropertyPackageManifest, StoredPropertyPackagePayload, StoredPropertyPackageSource,
    StoredThermoComponent, StoredThermoMethod, StoredVaporPhaseModel,
};
use rf_thermo::{
    AntoineCoefficients, LiquidPhaseModel, ThermoComponent, ThermoMethod, ThermoSystem,
    VaporPhaseModel,
};
use rf_types::RfResult;

pub(crate) fn property_package_manifest_from_stored(
    stored_manifest: StoredPropertyPackageManifest,
) -> RfResult<PropertyPackageManifest> {
    stored_manifest.validate()?;

    let source = property_package_source_from_stored(stored_manifest.source);
    let classification =
        property_package_classification_from_stored(stored_manifest.classification);

    let mut manifest = PropertyPackageManifest::new(
        stored_manifest.package_id,
        stored_manifest.version,
        source,
        stored_manifest.component_ids,
    );
    manifest.classification = classification;
    manifest.hash = stored_manifest.hash;
    manifest.size_bytes = stored_manifest.size_bytes;
    manifest.expires_at = stored_manifest.expires_at;
    Ok(manifest)
}

pub(crate) fn thermo_system_from_stored_payload(
    payload: StoredPropertyPackagePayload,
) -> ThermoSystem {
    let components = payload
        .components
        .into_iter()
        .map(thermo_component_from_stored)
        .collect();

    ThermoSystem {
        components,
        method: thermo_method_from_stored(payload.method),
    }
}

fn thermo_component_from_stored(component: StoredThermoComponent) -> ThermoComponent {
    ThermoComponent {
        id: component.id,
        name: component.name,
        antoine: component.antoine.map(antoine_coefficients_from_stored),
        liquid_heat_capacity_j_per_mol_k: component.liquid_heat_capacity_j_per_mol_k,
        vapor_heat_capacity_j_per_mol_k: component.vapor_heat_capacity_j_per_mol_k,
    }
}

fn antoine_coefficients_from_stored(
    coefficients: StoredAntoineCoefficients,
) -> AntoineCoefficients {
    AntoineCoefficients::new(coefficients.a, coefficients.b, coefficients.c)
}

fn thermo_method_from_stored(method: StoredThermoMethod) -> ThermoMethod {
    ThermoMethod {
        liquid_phase_model: liquid_phase_model_from_stored(method.liquid_phase_model),
        vapor_phase_model: vapor_phase_model_from_stored(method.vapor_phase_model),
    }
}

fn liquid_phase_model_from_stored(model: StoredLiquidPhaseModel) -> LiquidPhaseModel {
    match model {
        StoredLiquidPhaseModel::IdealSolution => LiquidPhaseModel::IdealSolution,
    }
}

fn vapor_phase_model_from_stored(model: StoredVaporPhaseModel) -> VaporPhaseModel {
    match model {
        StoredVaporPhaseModel::IdealGas => VaporPhaseModel::IdealGas,
    }
}

fn property_package_source_from_stored(
    source: StoredPropertyPackageSource,
) -> PropertyPackageSource {
    match source {
        StoredPropertyPackageSource::LocalBundled => PropertyPackageSource::LocalBundled,
        StoredPropertyPackageSource::RemoteDerivedPackage => {
            PropertyPackageSource::RemoteDerivedPackage
        }
        StoredPropertyPackageSource::RemoteEvaluationService => {
            PropertyPackageSource::RemoteEvaluationService
        }
    }
}

fn property_package_classification_from_stored(
    classification: StoredPropertyPackageClassification,
) -> PropertyPackageClassification {
    match classification {
        StoredPropertyPackageClassification::Derived => PropertyPackageClassification::Derived,
        StoredPropertyPackageClassification::RemoteOnly => {
            PropertyPackageClassification::RemoteOnly
        }
    }
}

pub(crate) fn manifest_to_stored(
    manifest: &PropertyPackageManifest,
) -> StoredPropertyPackageManifest {
    let source = match manifest.source {
        PropertyPackageSource::LocalBundled => StoredPropertyPackageSource::LocalBundled,
        PropertyPackageSource::RemoteDerivedPackage => {
            StoredPropertyPackageSource::RemoteDerivedPackage
        }
        PropertyPackageSource::RemoteEvaluationService => {
            StoredPropertyPackageSource::RemoteEvaluationService
        }
    };
    let mut stored = StoredPropertyPackageManifest::new(
        &manifest.package_id,
        &manifest.version,
        source,
        manifest.component_ids.clone(),
    );
    stored.classification = match manifest.classification {
        PropertyPackageClassification::Derived => StoredPropertyPackageClassification::Derived,
        PropertyPackageClassification::RemoteOnly => {
            StoredPropertyPackageClassification::RemoteOnly
        }
    };
    stored.hash = manifest.hash.clone();
    stored.size_bytes = manifest.size_bytes;
    stored.expires_at = manifest.expires_at;
    stored
}

pub(crate) fn system_to_stored(
    manifest: &PropertyPackageManifest,
    system: &ThermoSystem,
) -> StoredPropertyPackagePayload {
    let components = system
        .components
        .iter()
        .map(|component| StoredThermoComponent {
            id: component.id.clone(),
            name: component.name.clone(),
            antoine: component
                .antoine
                .as_ref()
                .map(|value| StoredAntoineCoefficients::new(value.a, value.b, value.c)),
            liquid_heat_capacity_j_per_mol_k: component.liquid_heat_capacity_j_per_mol_k,
            vapor_heat_capacity_j_per_mol_k: component.vapor_heat_capacity_j_per_mol_k,
        })
        .collect();
    let mut payload =
        StoredPropertyPackagePayload::new(&manifest.package_id, &manifest.version, components);
    payload.method = StoredThermoMethod {
        liquid_phase_model: match system.method.liquid_phase_model {
            LiquidPhaseModel::IdealSolution => StoredLiquidPhaseModel::IdealSolution,
        },
        vapor_phase_model: match system.method.vapor_phase_model {
            VaporPhaseModel::IdealGas => StoredVaporPhaseModel::IdealGas,
        },
    };
    payload
}
