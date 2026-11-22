//! Reviewed package and source binding for the first headless H3 flow case.

use super::{
    H3PureWaterQualification, H3WaterPropertySource, Side, qualification::bounds, rejected,
};
use rf_model::{
    engineering::{
        EngineeringBasis, SourceKind, SourceOperatingConditions, SourcePhase, SourceReview,
    },
    equipment::{
        ConcentricTubeLiquidDomain, ConcentricTubeLiquidProperties, ConcentricTubeRatingCase,
    },
};
use rf_property_package::{
    InMemoryPropertyPackageProvider, LoadedPropertyPackage, PackageDataOrigin,
};
use rf_store::{parse_property_package_manifest_json, parse_property_package_payload_json};
use rf_types::RfResult;
use std::time::UNIX_EPOCH;

pub(crate) const QUALIFICATION_ID: &str = "h3-pure-water";
pub(crate) const QUALIFICATION_REVISION: u64 = 1;
const PACKAGE_ID: &str = "h3-pure-water-lite-v1";
const PACKAGE_VERSION: &str = "1";
const PAYLOAD_SHA256: &str =
    "sha256:1d175d0a82c4d6f586219a32c1bdbad46555356e5d2d50eab3e6bd7a0cec0c26";
const PHASE_REFERENCE: &str = "https://webbook.nist.gov/cgi/cbook.cgi?ID=C7732185&Type=ANTOINE";
const PROPERTY_REFERENCE: &str = "https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=832373";
const LIQUID_DOMAIN: ConcentricTubeLiquidDomain = ConcentricTubeLiquidDomain {
    min_temperature_k: Some(293.15),
    max_temperature_k: Some(303.15),
    min_pressure_pa: Some(50_000.0),
    max_pressure_pa: Some(120_000.0),
};
const MANIFEST: &str = include_str!(
    "../../../../../examples/sample-components/property-packages/h3-pure-water-lite-v1/manifest.json"
);
const PAYLOAD: &str = include_str!(
    "../../../../../examples/sample-components/property-packages/h3-pure-water-lite-v1/payload.rfpkg"
);

pub(crate) fn qualification(id: &str, revision: u64) -> RfResult<H3PureWaterQualification> {
    if id != QUALIFICATION_ID || revision != QUALIFICATION_REVISION {
        return Err(rejected(
            "qualification_unavailable",
            "no bundled H3 qualification has the requested ID and revision",
        ));
    }
    Ok(reviewed_qualification())
}

pub(super) fn reviewed_qualification() -> H3PureWaterQualification {
    let source = H3WaterPropertySource {
        id: "nist-water-table-8".into(),
        revision: 3,
    };
    H3PureWaterQualification {
        component_id: "water".into(),
        package_id: PACKAGE_ID.into(),
        package_version: PACKAGE_VERSION.into(),
        payload_fingerprint: PAYLOAD_SHA256.into(),
        phase_reference: PHASE_REFERENCE.into(),
        phase_domain: LIQUID_DOMAIN,
        hot_properties: source.clone(),
        cold_properties: source,
    }
}

pub(crate) fn check_case(
    qualification: &H3PureWaterQualification,
    package_id: Option<&str>,
    basis: &EngineeringBasis,
    case: &ConcentricTubeRatingCase,
) -> RfResult<()> {
    if package_id != Some(qualification.package_id.as_str()) {
        return Err(rejected(
            "package_unqualified",
            "the saved flowsheet must select the package bound to the H3 qualification",
        ));
    }
    let association = case.flow_association.as_ref().ok_or_else(|| {
        rejected(
            "association_missing",
            "the H3 case has no saved flow association",
        )
    })?;
    for (side, stream, properties, approved) in [
        (
            Side::Hot,
            &association.hot.id,
            &case.hot.properties,
            &qualification.hot_properties,
        ),
        (
            Side::Cold,
            &association.cold.id,
            &case.cold.properties,
            &qualification.cold_properties,
        ),
    ] {
        let approved_source = basis.sources.iter().any(|record| {
            record.id == approved.id
                && record.revision == approved.revision
                && record.kind == SourceKind::Literature
                && record.review == SourceReview::Reviewed
                && record.reference == PROPERTY_REFERENCE
                && record.locator == "Table 8"
                && record.applicability.fluid_system == "Water"
                && record.applicability.phase == SourcePhase::Liquid
                && record.applicability.operating_conditions
                    == Some(SourceOperatingConditions {
                        temperature_k: 298.15,
                        pressure_pa: 100_000.0,
                    })
        });
        if !approved_source || properties.source_id != approved.id {
            return Err(side.reject(
                "property_source_unqualified",
                stream,
                "requires the reviewed revision and citation of the bundled liquid-water source",
            ));
        }
        let expected = ConcentricTubeLiquidProperties {
            source_id: approved.id.clone(),
            molar_mass_kg_per_mol: Some(0.018_015_28),
            molar_heat_capacity_j_per_mol_k: Some(75.329_923_737_630_4),
            density_kg_per_m3: Some(997.047_013),
            dynamic_viscosity_pa_s: Some(0.000_889_996_774),
            thermal_conductivity_w_per_m_k: Some(0.607_198_237),
            liquid_domain: properties.liquid_domain,
        };
        if *properties != expected
            || !bounds(properties.liquid_domain).is_some_and(|(t, p)| {
                let (approved_t, approved_p) =
                    bounds(LIQUID_DOMAIN).expect("reviewed water domain");
                approved_t.contains(t.start())
                    && approved_t.contains(t.end())
                    && approved_p.contains(p.start())
                    && approved_p.contains(p.end())
            })
        {
            return Err(side.reject("properties_unqualified", stream,
                "saved constants and their temperature/pressure domain must match the H3 qualification"));
        }
    }
    Ok(())
}

pub(crate) fn provider(
    qualification: &H3PureWaterQualification,
) -> RfResult<InMemoryPropertyPackageProvider> {
    let manifest = parse_property_package_manifest_json(MANIFEST)
        .map_err(|error| rejected("package_unqualified", error.to_string()))?;
    let payload = parse_property_package_payload_json(PAYLOAD)
        .map_err(|error| rejected("package_unqualified", error.to_string()))?;
    let package = LoadedPropertyPackage::from_stored(
        manifest,
        payload,
        PackageDataOrigin::Bundled,
        UNIX_EPOCH,
    )
    .map_err(|error| rejected("package_unqualified", error.to_string()))?;
    if !qualification.matches_package(
        &package.manifest().package_id,
        &package.manifest().version,
        &package.provenance().payload_fingerprint,
    ) {
        return Err(rejected(
            "package_unqualified",
            "bundled payload differs from the reviewed H3 qualification",
        ));
    }
    let mut provider = InMemoryPropertyPackageProvider::default();
    provider.insert(package);
    Ok(provider)
}
