use super::{Side, rejected};
use rf_model::{
    engineering::{EngineeringBasis, SourcePhase, SourceReview},
    equipment::{
        ConcentricTubeLiquidDomain, ConcentricTubeLiquidInlet, ConcentricTubeLiquidProperties,
    },
};
use rf_types::{ComponentId, PhaseEquilibriumRegion, PhaseLabel, RfResult, SourceId, StreamId};
use rf_ui::{StreamStateSnapshot, result_query::FlowResultOrigin};
use std::ops::RangeInclusive;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct H3WaterPropertySource {
    pub id: SourceId,
    pub revision: u64,
}

/// The caller supplies an independently reviewed pure-water phase basis and
/// the specific saved property sources it applies to. A converged flow solve
/// or a package label alone cannot grant this qualification.
#[derive(Debug, Clone)]
pub struct H3PureWaterQualification {
    pub component_id: ComponentId,
    pub package_id: String,
    pub package_version: String,
    pub payload_fingerprint: String,
    pub phase_reference: String,
    pub phase_domain: ConcentricTubeLiquidDomain,
    pub hot_properties: H3WaterPropertySource,
    pub cold_properties: H3WaterPropertySource,
}

impl H3PureWaterQualification {
    pub(super) fn validate(&self) -> RfResult<()> {
        if self.component_id.as_str().trim().is_empty()
            || self.package_id.trim().is_empty()
            || self.package_version.trim().is_empty()
            || !self
                .payload_fingerprint
                .strip_prefix("sha256:")
                .is_some_and(|hex| {
                    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
                })
            || self.phase_reference.trim().is_empty()
            || self.hot_properties.id.as_str().trim().is_empty()
            || self.cold_properties.id.as_str().trim().is_empty()
            || bounds(self.phase_domain).is_none()
        {
            return Err(rejected(
                "qualification_incomplete",
                "pure-water package, phase basis and both reviewed property sources are required",
            ));
        }
        Ok(())
    }

    pub(super) fn check_package(&self, origin: &FlowResultOrigin) -> RfResult<()> {
        let matches = origin
            .execution()
            .and_then(|execution| execution.package.as_ref())
            .is_some_and(|package| {
                package.package_id == self.package_id
                    && package.version == self.package_version
                    && package.payload_fingerprint == self.payload_fingerprint
            });
        if !matches {
            return Err(rejected(
                "package_unqualified",
                "the package actually used for this flow result lacks the reviewed pure-water qualification",
            ));
        }
        Ok(())
    }

    pub(super) fn inlet(
        &self,
        stream: &StreamStateSnapshot,
        properties: &ConcentricTubeLiquidProperties,
        basis: &EngineeringBasis,
        side: Side,
    ) -> RfResult<ConcentricTubeLiquidInlet> {
        let reject = |code, message| side.reject(code, &stream.stream_id, message);
        if !stream.total_molar_flow_mol_s.is_finite() || stream.total_molar_flow_mol_s <= 0.0 {
            return Err(reject(
                "invalid_flow",
                "requires a finite positive molar flow",
            ));
        }
        if !self.pure_water(&stream.overall_mole_fractions) {
            return Err(reject(
                "not_pure_water",
                "composition differs from the reviewed pure-water component",
            ));
        }
        let window = stream.bubble_dew_window.as_ref().ok_or_else(|| {
            reject(
                "phase_missing",
                "no phase-equilibrium window in the accepted result",
            )
        })?;
        if window.phase_region != PhaseEquilibriumRegion::LiquidOnly
            || !window.bubble_pressure_pa.is_finite()
            || window.bubble_pressure_pa <= 0.0
            || stream.pressure_pa <= window.bubble_pressure_pa
            || !stream
                .phases
                .iter()
                .all(|phase| match phase.label.as_str() {
                    label if label == PhaseLabel::Overall.as_str() => true,
                    label if label == PhaseLabel::Liquid.as_str() => {
                        phase.phase_fraction == 1.0 && self.pure_water(&phase.composition)
                    }
                    label if label == PhaseLabel::Vapor.as_str() => phase.phase_fraction == 0.0,
                    _ => false,
                })
        {
            return Err(reject(
                "not_single_liquid",
                "requires a subcooled single liquid phase",
            ));
        }
        if !in_domain(self.phase_domain, stream.temperature_k, stream.pressure_pa) {
            return Err(reject(
                "outside_phase_domain",
                "outside the reviewed phase temperature / pressure domain",
            ));
        }
        let approved = match side {
            Side::Hot => &self.hot_properties,
            Side::Cold => &self.cold_properties,
        };
        let reviewed = basis.sources.iter().any(|source| {
            source.id == approved.id
                && source.revision == approved.revision
                && source.review == SourceReview::Reviewed
                && source.applicability.phase == SourcePhase::Liquid
        });
        if properties.source_id != approved.id || !reviewed {
            return Err(reject(
                "property_source_unqualified",
                "requires the approved revision of the reviewed liquid property source",
            ));
        }
        if !in_domain(
            properties.liquid_domain,
            stream.temperature_k,
            stream.pressure_pa,
        ) {
            return Err(reject(
                "outside_property_domain",
                "outside the saved H3 liquid property temperature / pressure domain",
            ));
        }
        Ok(ConcentricTubeLiquidInlet {
            temperature_k: Some(stream.temperature_k),
            pressure_pa: Some(stream.pressure_pa),
            molar_flow_mol_s: Some(stream.total_molar_flow_mol_s),
        })
    }

    pub(super) fn check_outlet(
        &self,
        side: Side,
        stream: &StreamId,
        temperature_k: f64,
        pressure_pa: f64,
    ) -> RfResult<()> {
        if !in_domain(self.phase_domain, temperature_k, pressure_pa) {
            return Err(side.reject(
                "outlet_outside_phase_domain",
                stream,
                "rated outlet outside the reviewed phase temperature / pressure domain",
            ));
        }
        Ok(())
    }

    fn pure_water(&self, fractions: &[(String, f64)]) -> bool {
        let mut found_water = false;
        for (component, fraction) in fractions {
            if component == self.component_id.as_str() {
                if found_water || *fraction != 1.0 {
                    return false;
                }
                found_water = true;
            } else if *fraction != 0.0 {
                return false;
            }
        }
        found_water
    }
}

fn bounds(
    domain: ConcentricTubeLiquidDomain,
) -> Option<(RangeInclusive<f64>, RangeInclusive<f64>)> {
    fn interval(low: f64, high: f64) -> Option<RangeInclusive<f64>> {
        (low.is_finite() && high.is_finite() && low > 0.0 && low <= high).then_some(low..=high)
    }

    Some((
        interval(domain.min_temperature_k?, domain.max_temperature_k?)?,
        interval(domain.min_pressure_pa?, domain.max_pressure_pa?)?,
    ))
}

fn in_domain(domain: ConcentricTubeLiquidDomain, temperature_k: f64, pressure_pa: f64) -> bool {
    bounds(domain).is_some_and(|(temperature, pressure)| {
        temperature.contains(&temperature_k) && pressure.contains(&pressure_pa)
    })
}
