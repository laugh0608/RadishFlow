use std::collections::BTreeMap;
use std::time::SystemTime;

use rf_store::{
    StoredAntoineCoefficients, StoredAuthCacheIndex, StoredLiquidPhaseModel,
    StoredPropertyPackageClassification, StoredPropertyPackageManifest,
    StoredPropertyPackagePayload, StoredPropertyPackageSource, StoredThermoComponent,
    StoredThermoMethod, StoredVaporPhaseModel, read_property_package_manifest,
    read_property_package_payload,
};
use rf_types::{ComponentId, PhaseLabel, RfError, RfResult};

const REFERENCE_TEMPERATURE_K: f64 = 298.15;
const MOLE_FRACTION_SUM_TOLERANCE: f64 = 1e-9;
const PRESSURE_ORDER_TOLERANCE_PA: f64 = 1e-6;
const TEMPERATURE_ORDER_TOLERANCE_K: f64 = 1e-9;
const BOUNDARY_SOLVER_TEMPERATURE_TOLERANCE_K: f64 = 1e-9;
const BOUNDARY_SOLVER_PRESSURE_TOLERANCE_PA: f64 = 1e-6;

#[derive(Debug, Clone, PartialEq)]
pub struct AntoineCoefficients {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl AntoineCoefficients {
    pub fn new(a: f64, b: f64, c: f64) -> Self {
        Self { a, b, c }
    }

    pub fn saturation_pressure_pa(&self, temperature_k: f64) -> RfResult<f64> {
        if !temperature_k.is_finite() || temperature_k <= 0.0 {
            return Err(RfError::invalid_input(
                "temperature must be a finite number greater than zero kelvin",
            ));
        }

        let denominator = temperature_k + self.c;
        if !denominator.is_finite() || denominator.abs() <= f64::EPSILON {
            return Err(RfError::thermo(
                "Antoine correlation denominator is zero or non-finite",
            ));
        }

        // Current MVP property packages interpret Antoine coefficients as:
        // ln(P_sat / kPa) = A - B / (T[K] + C)
        let ln_pressure_kpa = self.a - (self.b / denominator);
        let saturation_pressure_pa = ln_pressure_kpa.exp() * 1_000.0;

        if !saturation_pressure_pa.is_finite() || saturation_pressure_pa <= 0.0 {
            return Err(RfError::thermo(
                "Antoine correlation produced a non-finite saturation pressure",
            ));
        }

        Ok(saturation_pressure_pa)
    }

    pub fn saturation_temperature_k(&self, pressure_pa: f64) -> RfResult<f64> {
        validate_pressure(pressure_pa)?;

        let pressure_kpa = pressure_pa / 1_000.0;
        let denominator = self.a - pressure_kpa.ln();
        if !denominator.is_finite() || denominator.abs() <= f64::EPSILON {
            return Err(RfError::thermo(
                "Antoine correlation cannot invert a zero or non-finite temperature denominator",
            ));
        }

        let temperature_k = (self.b / denominator) - self.c;
        if !temperature_k.is_finite() || temperature_k <= 0.0 {
            return Err(RfError::thermo(
                "Antoine correlation produced a non-finite saturation temperature",
            ));
        }

        Ok(temperature_k)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThermoComponent {
    pub id: ComponentId,
    pub name: String,
    pub antoine: Option<AntoineCoefficients>,
    pub liquid_heat_capacity_j_per_mol_k: Option<f64>,
    pub vapor_heat_capacity_j_per_mol_k: Option<f64>,
}

impl ThermoComponent {
    pub fn new(id: impl Into<ComponentId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            antoine: None,
            liquid_heat_capacity_j_per_mol_k: None,
            vapor_heat_capacity_j_per_mol_k: None,
        }
    }

    pub fn saturation_pressure_pa(&self, temperature_k: f64) -> RfResult<f64> {
        let antoine = self.antoine.as_ref().ok_or_else(|| {
            RfError::thermo(format!(
                "component `{}` is missing Antoine coefficients",
                self.id
            ))
        })?;

        antoine.saturation_pressure_pa(temperature_k)
    }

    pub fn saturation_temperature_k(&self, pressure_pa: f64) -> RfResult<f64> {
        let antoine = self.antoine.as_ref().ok_or_else(|| {
            RfError::thermo(format!(
                "component `{}` is missing Antoine coefficients",
                self.id
            ))
        })?;

        antoine.saturation_temperature_k(pressure_pa)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiquidPhaseModel {
    IdealSolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaporPhaseModel {
    IdealGas,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThermoMethod {
    pub liquid_phase_model: LiquidPhaseModel,
    pub vapor_phase_model: VaporPhaseModel,
}

impl Default for ThermoMethod {
    fn default() -> Self {
        Self {
            liquid_phase_model: LiquidPhaseModel::IdealSolution,
            vapor_phase_model: VaporPhaseModel::IdealGas,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThermoSystem {
    pub components: Vec<ThermoComponent>,
    pub method: ThermoMethod,
}

impl ThermoSystem {
    pub fn new(components: Vec<ThermoComponent>) -> Self {
        Self {
            components,
            method: ThermoMethod::default(),
        }
    }

    pub fn binary(components: [ThermoComponent; 2]) -> Self {
        Self::new(Vec::from(components))
    }

    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    pub fn component_ids(&self) -> Vec<ComponentId> {
        self.components
            .iter()
            .map(|component| component.id.clone())
            .collect()
    }

    pub fn validate_mole_fractions(&self, mole_fractions: &[f64]) -> RfResult<()> {
        if self.components.is_empty() {
            return Err(RfError::thermo(
                "thermo system must contain at least one component",
            ));
        }

        if mole_fractions.len() != self.component_count() {
            return Err(RfError::invalid_input(format!(
                "expected {} mole fractions, received {}",
                self.component_count(),
                mole_fractions.len()
            )));
        }

        if mole_fractions
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
        {
            return Err(RfError::invalid_input(
                "mole fractions must be finite non-negative values",
            ));
        }

        let sum = mole_fractions.iter().sum::<f64>();
        if !sum.is_finite() || sum <= 0.0 {
            return Err(RfError::invalid_input(
                "mole fractions must sum to a positive finite value",
            ));
        }

        if (sum - 1.0).abs() > MOLE_FRACTION_SUM_TOLERANCE {
            return Err(RfError::invalid_input(format!(
                "mole fractions must sum to one within tolerance {MOLE_FRACTION_SUM_TOLERANCE}, received {sum}"
            )));
        }

        Ok(())
    }

    pub fn validate_state(&self, state: &ThermoState) -> RfResult<()> {
        validate_temperature(state.temperature_k)?;
        validate_pressure(state.pressure_pa)?;

        self.validate_mole_fractions(&state.overall_mole_fractions)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThermoState {
    pub temperature_k: f64,
    pub pressure_pa: f64,
    pub overall_mole_fractions: Vec<f64>,
}

impl ThermoState {
    pub fn new(temperature_k: f64, pressure_pa: f64, overall_mole_fractions: Vec<f64>) -> Self {
        Self {
            temperature_k,
            pressure_pa,
            overall_mole_fractions,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BubbleDewPressureInput {
    pub temperature_k: f64,
    pub overall_mole_fractions: Vec<f64>,
}

impl BubbleDewPressureInput {
    pub fn new(temperature_k: f64, overall_mole_fractions: Vec<f64>) -> Self {
        Self {
            temperature_k,
            overall_mole_fractions,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BubbleDewPressures {
    pub bubble_pressure_pa: f64,
    pub dew_pressure_pa: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BubbleDewTemperatureInput {
    pub pressure_pa: f64,
    pub overall_mole_fractions: Vec<f64>,
}

impl BubbleDewTemperatureInput {
    pub fn new(pressure_pa: f64, overall_mole_fractions: Vec<f64>) -> Self {
        Self {
            pressure_pa,
            overall_mole_fractions,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BubbleDewTemperatures {
    pub bubble_temperature_k: f64,
    pub dew_temperature_k: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseThermoState {
    pub label: PhaseLabel,
    pub temperature_k: f64,
    pub pressure_pa: f64,
    pub mole_fractions: Vec<f64>,
}

impl PhaseThermoState {
    pub fn new(
        label: PhaseLabel,
        temperature_k: f64,
        pressure_pa: f64,
        mole_fractions: Vec<f64>,
    ) -> Self {
        Self {
            label,
            temperature_k,
            pressure_pa,
            mole_fractions,
        }
    }
}

pub trait ThermoProvider {
    fn system(&self) -> &ThermoSystem;

    fn estimate_k_values(&self, state: &ThermoState) -> RfResult<Vec<f64>>;

    fn estimate_bubble_dew_pressures(
        &self,
        input: &BubbleDewPressureInput,
    ) -> RfResult<BubbleDewPressures>;

    fn estimate_bubble_dew_temperatures(
        &self,
        input: &BubbleDewTemperatureInput,
    ) -> RfResult<BubbleDewTemperatures>;

    fn phase_molar_enthalpy(&self, state: &PhaseThermoState) -> RfResult<f64>;
}

#[derive(Debug, Clone)]
pub struct PlaceholderThermoProvider {
    system: ThermoSystem,
}

impl PlaceholderThermoProvider {
    pub fn new(system: ThermoSystem) -> Self {
        Self { system }
    }

    fn saturation_pressures_pa(&self, temperature_k: f64) -> RfResult<Vec<f64>> {
        validate_temperature(temperature_k)?;

        self.system
            .components
            .iter()
            .map(|component| component.saturation_pressure_pa(temperature_k))
            .collect()
    }

    fn saturation_temperatures_k(&self, pressure_pa: f64) -> RfResult<Vec<f64>> {
        validate_pressure(pressure_pa)?;

        self.system
            .components
            .iter()
            .map(|component| component.saturation_temperature_k(pressure_pa))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyPackageSource {
    LocalBundled,
    RemoteDerivedPackage,
    RemoteEvaluationService,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyPackageClassification {
    Derived,
    RemoteOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyPackageManifest {
    pub package_id: String,
    pub version: String,
    pub classification: PropertyPackageClassification,
    pub source: PropertyPackageSource,
    pub hash: String,
    pub size_bytes: u64,
    pub component_ids: Vec<ComponentId>,
    pub expires_at: Option<SystemTime>,
}

impl PropertyPackageManifest {
    pub fn new(
        package_id: impl Into<String>,
        version: impl Into<String>,
        source: PropertyPackageSource,
        component_ids: Vec<ComponentId>,
    ) -> Self {
        let classification = match source {
            PropertyPackageSource::RemoteEvaluationService => {
                PropertyPackageClassification::RemoteOnly
            }
            PropertyPackageSource::LocalBundled | PropertyPackageSource::RemoteDerivedPackage => {
                PropertyPackageClassification::Derived
            }
        };

        Self {
            package_id: package_id.into(),
            version: version.into(),
            classification,
            source,
            hash: String::new(),
            size_bytes: 0,
            component_ids,
            expires_at: None,
        }
    }
}

pub trait PropertyPackageProvider {
    fn list_manifests(&self) -> Vec<PropertyPackageManifest>;

    fn load_system(&self, package_id: &str) -> RfResult<ThermoSystem>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryPropertyPackageProvider {
    packages: BTreeMap<String, (PropertyPackageManifest, ThermoSystem)>,
}

impl InMemoryPropertyPackageProvider {
    pub fn new(entries: Vec<(PropertyPackageManifest, ThermoSystem)>) -> Self {
        let packages = entries
            .into_iter()
            .map(|(manifest, system)| (manifest.package_id.clone(), (manifest, system)))
            .collect();

        Self { packages }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CachedPropertyPackageProvider {
    packages: BTreeMap<String, (PropertyPackageManifest, ThermoSystem)>,
}

impl CachedPropertyPackageProvider {
    pub fn new(
        cache_root: impl AsRef<std::path::Path>,
        auth_cache_index: &StoredAuthCacheIndex,
    ) -> RfResult<Self> {
        Self::new_at(cache_root, auth_cache_index, SystemTime::now())
    }

    pub fn new_at(
        cache_root: impl AsRef<std::path::Path>,
        auth_cache_index: &StoredAuthCacheIndex,
        now: SystemTime,
    ) -> RfResult<Self> {
        auth_cache_index.validate()?;

        let cache_root = cache_root.as_ref();
        let mut packages = BTreeMap::new();

        for record in &auth_cache_index.property_packages {
            if record.is_expired_at(now) {
                continue;
            }

            if matches!(
                record.source,
                StoredPropertyPackageSource::RemoteEvaluationService
            ) {
                continue;
            }

            let manifest_path = record.manifest_path_under(cache_root);
            let stored_manifest = read_property_package_manifest(&manifest_path)?;
            stored_manifest.validate_against_record(record)?;

            let payload_path = record.payload_path_under(cache_root).ok_or_else(|| {
                RfError::invalid_input(format!(
                    "stored property package `{}` is missing a local payload path",
                    record.package_id
                ))
            })?;
            let stored_payload = read_property_package_payload(&payload_path)?;
            stored_payload.validate_against_manifest(&stored_manifest)?;

            let runtime_manifest = property_package_manifest_from_stored(stored_manifest)?;
            let runtime_system = thermo_system_from_stored_payload(stored_payload);
            let package_id = runtime_manifest.package_id.clone();

            if packages
                .insert(package_id.clone(), (runtime_manifest, runtime_system))
                .is_some()
            {
                return Err(RfError::invalid_input(format!(
                    "duplicate cached property package `{package_id}` found in auth cache index"
                )));
            }
        }

        Ok(Self { packages })
    }
}

impl PropertyPackageProvider for InMemoryPropertyPackageProvider {
    fn list_manifests(&self) -> Vec<PropertyPackageManifest> {
        self.packages
            .values()
            .map(|(manifest, _)| manifest.clone())
            .collect()
    }

    fn load_system(&self, package_id: &str) -> RfResult<ThermoSystem> {
        self.packages
            .get(package_id)
            .map(|(_, system)| system.clone())
            .ok_or_else(|| RfError::missing_entity("property package", package_id))
    }
}

impl PropertyPackageProvider for CachedPropertyPackageProvider {
    fn list_manifests(&self) -> Vec<PropertyPackageManifest> {
        self.packages
            .values()
            .map(|(manifest, _)| manifest.clone())
            .collect()
    }

    fn load_system(&self, package_id: &str) -> RfResult<ThermoSystem> {
        self.packages
            .get(package_id)
            .map(|(_, system)| system.clone())
            .ok_or_else(|| RfError::missing_entity("property package", package_id))
    }
}

impl ThermoProvider for PlaceholderThermoProvider {
    fn system(&self) -> &ThermoSystem {
        &self.system
    }

    fn estimate_k_values(&self, state: &ThermoState) -> RfResult<Vec<f64>> {
        self.system.validate_state(state)?;
        let saturation_pressures = self.saturation_pressures_pa(state.temperature_k)?;

        saturation_pressures
            .into_iter()
            .map(|saturation_pressure_pa| Ok(saturation_pressure_pa / state.pressure_pa))
            .collect()
    }

    fn estimate_bubble_dew_pressures(
        &self,
        input: &BubbleDewPressureInput,
    ) -> RfResult<BubbleDewPressures> {
        validate_temperature(input.temperature_k)?;
        self.system
            .validate_mole_fractions(&input.overall_mole_fractions)?;

        let saturation_pressures = self.saturation_pressures_pa(input.temperature_k)?;
        let bubble_pressure_pa = bubble_pressure_from_saturation_pressures(
            &input.overall_mole_fractions,
            &saturation_pressures,
        )?;
        let dew_pressure_pa = dew_pressure_from_saturation_pressures(
            &input.overall_mole_fractions,
            &saturation_pressures,
        )?;

        if bubble_pressure_pa + PRESSURE_ORDER_TOLERANCE_PA < dew_pressure_pa {
            return Err(RfError::thermo(
                "bubble pressure cannot be lower than dew pressure for the same mixture state",
            ));
        }

        Ok(BubbleDewPressures {
            bubble_pressure_pa,
            dew_pressure_pa,
        })
    }

    fn estimate_bubble_dew_temperatures(
        &self,
        input: &BubbleDewTemperatureInput,
    ) -> RfResult<BubbleDewTemperatures> {
        validate_pressure(input.pressure_pa)?;
        self.system
            .validate_mole_fractions(&input.overall_mole_fractions)?;

        let saturation_temperatures = self.saturation_temperatures_k(input.pressure_pa)?;
        let (lower_temperature_k, upper_temperature_k) =
            pure_temperature_bracket(&saturation_temperatures)?;

        let bubble_temperature_k = solve_boundary_temperature(
            lower_temperature_k,
            upper_temperature_k,
            input.pressure_pa,
            "bubble",
            |temperature_k| {
                let saturation_pressures = self.saturation_pressures_pa(temperature_k)?;
                bubble_pressure_from_saturation_pressures(
                    &input.overall_mole_fractions,
                    &saturation_pressures,
                )
            },
        )?;
        let dew_temperature_k = solve_boundary_temperature(
            lower_temperature_k,
            upper_temperature_k,
            input.pressure_pa,
            "dew",
            |temperature_k| {
                let saturation_pressures = self.saturation_pressures_pa(temperature_k)?;
                dew_pressure_from_saturation_pressures(
                    &input.overall_mole_fractions,
                    &saturation_pressures,
                )
            },
        )?;

        if bubble_temperature_k - dew_temperature_k > TEMPERATURE_ORDER_TOLERANCE_K {
            return Err(RfError::thermo(
                "bubble temperature cannot exceed dew temperature for the same mixture state",
            ));
        }

        Ok(BubbleDewTemperatures {
            bubble_temperature_k,
            dew_temperature_k,
        })
    }

    fn phase_molar_enthalpy(&self, state: &PhaseThermoState) -> RfResult<f64> {
        validate_temperature(state.temperature_k)?;
        validate_pressure(state.pressure_pa)?;

        self.system.validate_mole_fractions(&state.mole_fractions)?;

        let heat_capacity = self
            .system
            .components
            .iter()
            .zip(state.mole_fractions.iter())
            .try_fold(0.0, |total, (component, fraction)| {
                let capacity = component_heat_capacity(component, state.label)?;
                Ok(total + fraction * capacity)
            })?;

        Ok(heat_capacity * (state.temperature_k - REFERENCE_TEMPERATURE_K))
    }
}

fn component_heat_capacity(component: &ThermoComponent, label: PhaseLabel) -> RfResult<f64> {
    let capacity = match label {
        PhaseLabel::Liquid => component.liquid_heat_capacity_j_per_mol_k,
        PhaseLabel::Vapor => component.vapor_heat_capacity_j_per_mol_k,
        PhaseLabel::Overall => {
            return Err(RfError::thermo(
                "phase enthalpy requires a concrete liquid or vapor phase label",
            ));
        }
    }
    .ok_or_else(|| {
        RfError::thermo(format!(
            "component `{}` is missing `{}` heat capacity",
            component.id, label
        ))
    })?;

    if !capacity.is_finite() || capacity <= 0.0 {
        return Err(RfError::thermo(format!(
            "component `{}` has a non-positive or non-finite `{}` heat capacity",
            component.id, label
        )));
    }

    Ok(capacity)
}

fn bubble_pressure_from_saturation_pressures(
    mole_fractions: &[f64],
    saturation_pressures: &[f64],
) -> RfResult<f64> {
    let bubble_pressure_pa = mole_fractions
        .iter()
        .zip(saturation_pressures.iter())
        .map(|(fraction, saturation_pressure_pa)| fraction * saturation_pressure_pa)
        .sum::<f64>();
    if !bubble_pressure_pa.is_finite() || bubble_pressure_pa <= 0.0 {
        return Err(RfError::thermo(
            "bubble pressure estimate produced a non-finite pressure",
        ));
    }

    Ok(bubble_pressure_pa)
}

fn dew_pressure_from_saturation_pressures(
    mole_fractions: &[f64],
    saturation_pressures: &[f64],
) -> RfResult<f64> {
    let dew_denominator = mole_fractions
        .iter()
        .zip(saturation_pressures.iter())
        .map(|(fraction, saturation_pressure_pa)| fraction / saturation_pressure_pa)
        .sum::<f64>();
    if !dew_denominator.is_finite() || dew_denominator <= 0.0 {
        return Err(RfError::thermo(
            "dew pressure estimate produced a non-finite denominator",
        ));
    }

    let dew_pressure_pa = 1.0 / dew_denominator;
    if !dew_pressure_pa.is_finite() || dew_pressure_pa <= 0.0 {
        return Err(RfError::thermo(
            "dew pressure estimate produced a non-finite pressure",
        ));
    }

    Ok(dew_pressure_pa)
}

fn pure_temperature_bracket(saturation_temperatures: &[f64]) -> RfResult<(f64, f64)> {
    let mut lower_temperature_k = f64::INFINITY;
    let mut upper_temperature_k = f64::NEG_INFINITY;

    for temperature_k in saturation_temperatures {
        if !temperature_k.is_finite() || *temperature_k <= 0.0 {
            return Err(RfError::thermo(
                "pure-component saturation temperature bracket contains a non-finite temperature",
            ));
        }

        lower_temperature_k = lower_temperature_k.min(*temperature_k);
        upper_temperature_k = upper_temperature_k.max(*temperature_k);
    }

    if !lower_temperature_k.is_finite()
        || !upper_temperature_k.is_finite()
        || lower_temperature_k <= 0.0
        || upper_temperature_k <= 0.0
    {
        return Err(RfError::thermo(
            "could not derive a finite pure-component temperature bracket",
        ));
    }

    Ok((lower_temperature_k, upper_temperature_k))
}

fn solve_boundary_temperature(
    lower_temperature_k: f64,
    upper_temperature_k: f64,
    target_pressure_pa: f64,
    label: &str,
    estimate_pressure: impl Fn(f64) -> RfResult<f64>,
) -> RfResult<f64> {
    let mut lower = lower_temperature_k;
    let mut upper = upper_temperature_k;
    let mut lower_delta = estimate_pressure(lower)? - target_pressure_pa;
    if !lower_delta.is_finite() {
        return Err(RfError::thermo(format!(
            "{label} temperature estimate produced a non-finite lower bracket value"
        )));
    }

    let mut upper_delta = estimate_pressure(upper)? - target_pressure_pa;
    if !upper_delta.is_finite() {
        return Err(RfError::thermo(format!(
            "{label} temperature estimate produced a non-finite upper bracket value"
        )));
    }

    if lower_delta.abs() <= BOUNDARY_SOLVER_PRESSURE_TOLERANCE_PA {
        return Ok(lower);
    }

    if upper_delta.abs() <= BOUNDARY_SOLVER_PRESSURE_TOLERANCE_PA {
        return Ok(upper);
    }

    if lower_delta > BOUNDARY_SOLVER_PRESSURE_TOLERANCE_PA
        || upper_delta < -BOUNDARY_SOLVER_PRESSURE_TOLERANCE_PA
    {
        return Err(RfError::thermo(format!(
            "{label} temperature estimate could not bracket the target pressure `{target_pressure_pa}` Pa"
        )));
    }

    for _ in 0..100 {
        let midpoint = 0.5 * (lower + upper);
        let midpoint_delta = estimate_pressure(midpoint)? - target_pressure_pa;
        if !midpoint_delta.is_finite() {
            return Err(RfError::thermo(format!(
                "{label} temperature estimate produced a non-finite iteration value"
            )));
        }

        if midpoint_delta.abs() <= BOUNDARY_SOLVER_PRESSURE_TOLERANCE_PA
            || (upper - lower).abs() <= BOUNDARY_SOLVER_TEMPERATURE_TOLERANCE_K
        {
            return Ok(midpoint);
        }

        if midpoint_delta < 0.0 {
            lower = midpoint;
            lower_delta = midpoint_delta;
        } else {
            upper = midpoint;
            upper_delta = midpoint_delta;
        }
    }

    let midpoint = 0.5 * (lower + upper);
    let midpoint_delta = estimate_pressure(midpoint)? - target_pressure_pa;
    if midpoint_delta.abs() > BOUNDARY_SOLVER_PRESSURE_TOLERANCE_PA
        && (upper - lower).abs() > BOUNDARY_SOLVER_TEMPERATURE_TOLERANCE_K
    {
        return Err(RfError::thermo(format!(
            "{label} temperature estimate did not converge within the boundary solver tolerance"
        )));
    }

    let _ = lower_delta;
    let _ = upper_delta;
    Ok(midpoint)
}

fn validate_temperature(temperature_k: f64) -> RfResult<()> {
    if !temperature_k.is_finite() || temperature_k <= 0.0 {
        return Err(RfError::invalid_input(
            "temperature must be a finite value greater than zero kelvin",
        ));
    }

    Ok(())
}

fn validate_pressure(pressure_pa: f64) -> RfResult<()> {
    if !pressure_pa.is_finite() || pressure_pa <= 0.0 {
        return Err(RfError::invalid_input(
            "pressure must be a finite value greater than zero pascal",
        ));
    }

    Ok(())
}

fn property_package_manifest_from_stored(
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

fn thermo_system_from_stored_payload(payload: StoredPropertyPackagePayload) -> ThermoSystem {
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use rf_store::{
        StoredAuthCacheIndex, StoredCredentialReference, StoredPropertyPackageManifest,
        StoredPropertyPackagePayload, StoredPropertyPackageRecord, StoredPropertyPackageSource,
        StoredThermoComponent, property_package_payload_integrity, read_property_package_manifest,
        read_property_package_payload, write_property_package_manifest,
        write_property_package_payload,
    };

    use super::{
        AntoineCoefficients, BubbleDewPressureInput, BubbleDewTemperatureInput,
        CachedPropertyPackageProvider, InMemoryPropertyPackageProvider, PhaseThermoState,
        PlaceholderThermoProvider, PropertyPackageClassification, PropertyPackageManifest,
        PropertyPackageProvider, PropertyPackageSource, ThermoComponent, ThermoProvider,
        ThermoState, ThermoSystem,
    };
    use rf_types::{
        ComponentId, PhaseEquilibriumRegion, PhaseLabel, phase_equilibrium_region_from_pressure,
        phase_equilibrium_region_from_temperature,
    };

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        let delta = (actual - expected).abs();
        assert!(
            delta <= tolerance,
            "expected {actual} to be within {tolerance} of {expected}, delta was {delta}"
        );
    }

    fn build_binary_hydrocarbon_lite_provider() -> PlaceholderThermoProvider {
        let mut methane = ThermoComponent::new(ComponentId::new("methane"), "Methane");
        methane.antoine = Some(AntoineCoefficients::new(8.987, 659.7, -16.7));
        methane.liquid_heat_capacity_j_per_mol_k = Some(35.0);
        methane.vapor_heat_capacity_j_per_mol_k = Some(36.5);

        let mut ethane = ThermoComponent::new(ComponentId::new("ethane"), "Ethane");
        ethane.antoine = Some(AntoineCoefficients::new(8.952, 699.7, -22.8));
        ethane.liquid_heat_capacity_j_per_mol_k = Some(52.0);
        ethane.vapor_heat_capacity_j_per_mol_k = Some(65.0);

        PlaceholderThermoProvider::new(ThermoSystem::binary([methane, ethane]))
    }

    fn build_test_antoine_coefficients(k_value: f64, pressure_pa: f64) -> AntoineCoefficients {
        const TEST_ANTOINE_BOUNDARY_SLOPE: f64 = 250.0;
        const TEST_REFERENCE_TEMPERATURE_K: f64 = 300.0;

        AntoineCoefficients::new(
            ((k_value * pressure_pa) / 1_000.0).ln()
                + TEST_ANTOINE_BOUNDARY_SLOPE / TEST_REFERENCE_TEMPERATURE_K,
            TEST_ANTOINE_BOUNDARY_SLOPE,
            0.0,
        )
    }

    fn build_synthetic_provider(k_values: [f64; 2], pressure_pa: f64) -> PlaceholderThermoProvider {
        let mut first = ThermoComponent::new(ComponentId::new("component-a"), "Component A");
        first.antoine = Some(build_test_antoine_coefficients(k_values[0], pressure_pa));
        first.liquid_heat_capacity_j_per_mol_k = Some(35.0);
        first.vapor_heat_capacity_j_per_mol_k = Some(36.5);

        let mut second = ThermoComponent::new(ComponentId::new("component-b"), "Component B");
        second.antoine = Some(build_test_antoine_coefficients(k_values[1], pressure_pa));
        second.liquid_heat_capacity_j_per_mol_k = Some(52.0);
        second.vapor_heat_capacity_j_per_mol_k = Some(65.0);

        PlaceholderThermoProvider::new(ThermoSystem::binary([first, second]))
    }

    struct ThermoPressureBoundaryPerturbationCase {
        label: &'static str,
        pressure_pa: f64,
        expected_region: PhaseEquilibriumRegion,
        expected_bubble_temperature_k: f64,
        expected_dew_temperature_k: f64,
    }

    struct ThermoTemperatureBoundaryPerturbationCase {
        label: &'static str,
        temperature_k: f64,
        expected_region: PhaseEquilibriumRegion,
        expected_bubble_pressure_pa: f64,
        expected_dew_pressure_pa: f64,
    }

    fn assert_pressure_boundary_perturbations_keep_window_consistent(
        provider: &dyn ThermoProvider,
        overall_mole_fractions: &[f64],
        temperature_k: f64,
        exact_bubble_pressure_pa: f64,
        exact_dew_pressure_pa: f64,
        cases: &[ThermoPressureBoundaryPerturbationCase],
    ) {
        for case in cases {
            let pressures = provider
                .estimate_bubble_dew_pressures(&BubbleDewPressureInput::new(
                    temperature_k,
                    overall_mole_fractions.to_vec(),
                ))
                .expect("expected pressure window");
            let temperatures = provider
                .estimate_bubble_dew_temperatures(&BubbleDewTemperatureInput::new(
                    case.pressure_pa,
                    overall_mole_fractions.to_vec(),
                ))
                .expect("expected temperature window");

            assert_close(pressures.bubble_pressure_pa, exact_bubble_pressure_pa, 1e-6);
            assert_close(pressures.dew_pressure_pa, exact_dew_pressure_pa, 1e-6);
            assert_close(
                temperatures.bubble_temperature_k,
                case.expected_bubble_temperature_k,
                1e-4,
            );
            assert_close(
                temperatures.dew_temperature_k,
                case.expected_dew_temperature_k,
                1e-4,
            );

            let pressure_region = phase_equilibrium_region_from_pressure(
                case.pressure_pa,
                pressures.bubble_pressure_pa,
                pressures.dew_pressure_pa,
            );
            let temperature_region = phase_equilibrium_region_from_temperature(
                temperature_k,
                temperatures.bubble_temperature_k,
                temperatures.dew_temperature_k,
            );

            assert_eq!(pressure_region, case.expected_region, "{}", case.label);
            assert_eq!(temperature_region, case.expected_region, "{}", case.label);
        }
    }

    fn assert_temperature_boundary_perturbations_keep_window_consistent(
        provider: &dyn ThermoProvider,
        overall_mole_fractions: &[f64],
        pressure_pa: f64,
        exact_bubble_temperature_k: f64,
        exact_dew_temperature_k: f64,
        cases: &[ThermoTemperatureBoundaryPerturbationCase],
    ) {
        for case in cases {
            let pressures = provider
                .estimate_bubble_dew_pressures(&BubbleDewPressureInput::new(
                    case.temperature_k,
                    overall_mole_fractions.to_vec(),
                ))
                .expect("expected pressure window");
            let temperatures = provider
                .estimate_bubble_dew_temperatures(&BubbleDewTemperatureInput::new(
                    pressure_pa,
                    overall_mole_fractions.to_vec(),
                ))
                .expect("expected temperature window");

            assert_close(
                pressures.bubble_pressure_pa,
                case.expected_bubble_pressure_pa,
                1e-6,
            );
            assert_close(
                pressures.dew_pressure_pa,
                case.expected_dew_pressure_pa,
                1e-6,
            );
            assert_close(
                temperatures.bubble_temperature_k,
                exact_bubble_temperature_k,
                1e-4,
            );
            assert_close(
                temperatures.dew_temperature_k,
                exact_dew_temperature_k,
                1e-4,
            );

            let pressure_region = phase_equilibrium_region_from_pressure(
                pressure_pa,
                pressures.bubble_pressure_pa,
                pressures.dew_pressure_pa,
            );
            let temperature_region = phase_equilibrium_region_from_temperature(
                case.temperature_k,
                temperatures.bubble_temperature_k,
                temperatures.dew_temperature_k,
            );

            assert_eq!(pressure_region, case.expected_region, "{}", case.label);
            assert_eq!(temperature_region, case.expected_region, "{}", case.label);
        }
    }

    struct BinaryHydrocarbonLiteBoundaryScenario {
        overall_mole_fractions: [f64; 2],
        exact_bubble_pressure_pa: f64,
        exact_dew_pressure_pa: f64,
        exact_bubble_temperature_k: f64,
        exact_dew_temperature_k: f64,
        pressure_cases: Vec<ThermoPressureBoundaryPerturbationCase>,
        temperature_cases: Vec<ThermoTemperatureBoundaryPerturbationCase>,
    }

    fn binary_hydrocarbon_lite_two_phase_boundary_scenarios()
    -> Vec<BinaryHydrocarbonLiteBoundaryScenario> {
        vec![
            BinaryHydrocarbonLiteBoundaryScenario {
                overall_mole_fractions: [0.195, 0.805],
                exact_bubble_pressure_pa: 650_117.7234978296,
                exact_dew_pressure_pa: 644_714.832888367,
                exact_bubble_temperature_k: 299.9796507687297,
                exact_dew_temperature_k: 300.9138867482865,
                pressure_cases: vec![
                    ThermoPressureBoundaryPerturbationCase {
                        label: "z=[0.195, 0.805] bubble-boundary - 0.1 Pa",
                        pressure_pa: 650_117.6234978297,
                        expected_region: PhaseEquilibriumRegion::TwoPhase,
                        expected_bubble_temperature_k: 299.99998271468905,
                        expected_dew_temperature_k: 300.9342091831396,
                    },
                    ThermoPressureBoundaryPerturbationCase {
                        label: "z=[0.195, 0.805] bubble-boundary + 0.1 Pa",
                        pressure_pa: 650_117.8234978296,
                        expected_region: PhaseEquilibriumRegion::LiquidOnly,
                        expected_bubble_temperature_k: 300.00001728531026,
                        expected_dew_temperature_k: 300.9342437375931,
                    },
                    ThermoPressureBoundaryPerturbationCase {
                        label: "z=[0.195, 0.805] dew-boundary + 0.1 Pa",
                        pressure_pa: 644_714.932888367,
                        expected_region: PhaseEquilibriumRegion::TwoPhase,
                        expected_bubble_temperature_k: 299.06534905353476,
                        expected_dew_temperature_k: 300.00001730535257,
                    },
                    ThermoPressureBoundaryPerturbationCase {
                        label: "z=[0.195, 0.805] dew-boundary - 0.1 Pa",
                        pressure_pa: 644_714.732888367,
                        expected_region: PhaseEquilibriumRegion::VaporOnly,
                        expected_bubble_temperature_k: 299.0653144262868,
                        expected_dew_temperature_k: 299.99998269464675,
                    },
                ],
                temperature_cases: vec![
                    ThermoTemperatureBoundaryPerturbationCase {
                        label: "z=[0.195, 0.805] bubble-temperature - 0.001 K",
                        temperature_k: 299.9786507687297,
                        expected_region: PhaseEquilibriumRegion::LiquidOnly,
                        expected_bubble_pressure_pa: 649_994.2149500577,
                        expected_dew_pressure_pa: 644_591.4674174292,
                    },
                    ThermoTemperatureBoundaryPerturbationCase {
                        label: "z=[0.195, 0.805] bubble-temperature + 0.001 K",
                        temperature_k: 299.9806507687297,
                        expected_region: PhaseEquilibriumRegion::TwoPhase,
                        expected_bubble_pressure_pa: 650_005.7850599772,
                        expected_dew_pressure_pa: 644_603.0241212061,
                    },
                    ThermoTemperatureBoundaryPerturbationCase {
                        label: "z=[0.195, 0.805] dew-temperature - 0.001 K",
                        temperature_k: 300.9128867482865,
                        expected_region: PhaseEquilibriumRegion::TwoPhase,
                        expected_bubble_pressure_pa: 655_403.161968214,
                        expected_dew_pressure_pa: 649_994.2122419368,
                    },
                    ThermoTemperatureBoundaryPerturbationCase {
                        label: "z=[0.195, 0.805] dew-temperature + 0.001 K",
                        temperature_k: 300.91488674828645,
                        expected_region: PhaseEquilibriumRegion::VaporOnly,
                        expected_bubble_pressure_pa: 655_414.7506432333,
                        expected_dew_pressure_pa: 650_005.7877680402,
                    },
                ],
            },
            BinaryHydrocarbonLiteBoundaryScenario {
                overall_mole_fractions: [0.2, 0.8],
                exact_bubble_pressure_pa: 650_919.9866646,
                exact_dew_pressure_pa: 645_407.066294851,
                exact_bubble_temperature_k: 299.8410613926369,
                exact_dew_temperature_k: 300.79375964816904,
                pressure_cases: vec![
                    ThermoPressureBoundaryPerturbationCase {
                        label: "z=[0.2, 0.8] bubble-boundary - 0.1 Pa",
                        pressure_pa: 650_919.8866645998,
                        expected_region: PhaseEquilibriumRegion::TwoPhase,
                        expected_bubble_temperature_k: 299.9999827261904,
                        expected_dew_temperature_k: 300.95260505288763,
                    },
                    ThermoPressureBoundaryPerturbationCase {
                        label: "z=[0.2, 0.8] bubble-boundary + 0.1 Pa",
                        pressure_pa: 650_920.0866645997,
                        expected_region: PhaseEquilibriumRegion::LiquidOnly,
                        expected_bubble_temperature_k: 300.0000172736834,
                        expected_dew_temperature_k: 300.9526395843402,
                    },
                    ThermoPressureBoundaryPerturbationCase {
                        label: "z=[0.2, 0.8] dew-boundary + 0.1 Pa",
                        pressure_pa: 645_407.1662948506,
                        expected_region: PhaseEquilibriumRegion::TwoPhase,
                        expected_bubble_temperature_k: 299.04693549691126,
                        expected_dew_temperature_k: 300.0000172943121,
                    },
                    ThermoPressureBoundaryPerturbationCase {
                        label: "z=[0.2, 0.8] dew-boundary - 0.1 Pa",
                        pressure_pa: 645_406.9662948507,
                        expected_region: PhaseEquilibriumRegion::VaporOnly,
                        expected_bubble_temperature_k: 299.04690089204394,
                        expected_dew_temperature_k: 299.9999827057845,
                    },
                ],
                temperature_cases: vec![
                    ThermoTemperatureBoundaryPerturbationCase {
                        label: "z=[0.2, 0.8] bubble-temperature - 0.001 K",
                        temperature_k: 299.8400613926369,
                        expected_region: PhaseEquilibriumRegion::LiquidOnly,
                        expected_bubble_pressure_pa: 649_994.2124871389,
                        expected_dew_pressure_pa: 644_482.3840045808,
                    },
                    ThermoTemperatureBoundaryPerturbationCase {
                        label: "z=[0.2, 0.8] bubble-temperature + 0.001 K",
                        temperature_k: 299.8420613926369,
                        expected_region: PhaseEquilibriumRegion::TwoPhase,
                        expected_bubble_pressure_pa: 650_005.7875219034,
                        expected_dew_pressure_pa: 644_493.9453632077,
                    },
                    ThermoTemperatureBoundaryPerturbationCase {
                        label: "z=[0.2, 0.8] dew-temperature - 0.001 K",
                        temperature_k: 300.79275964816907,
                        expected_region: PhaseEquilibriumRegion::TwoPhase,
                        expected_bubble_pressure_pa: 655_512.4890424822,
                        expected_dew_pressure_pa: 649_994.2097160413,
                    },
                    ThermoTemperatureBoundaryPerturbationCase {
                        label: "z=[0.2, 0.8] dew-temperature + 0.001 K",
                        temperature_k: 300.794759648169,
                        expected_region: PhaseEquilibriumRegion::VaporOnly,
                        expected_bubble_pressure_pa: 655_524.0830284748,
                        expected_dew_pressure_pa: 650_005.7902937229,
                    },
                ],
            },
            BinaryHydrocarbonLiteBoundaryScenario {
                overall_mole_fractions: [0.23, 0.77],
                exact_bubble_pressure_pa: 655_733.5656652204,
                exact_dew_pressure_pa: 649_591.885824445,
                exact_bubble_temperature_k: 299.01269611750297,
                exact_dew_temperature_k: 300.07030238460186,
                pressure_cases: vec![
                    ThermoPressureBoundaryPerturbationCase {
                        label: "z=[0.23, 0.77] bubble-boundary - 0.1 Pa",
                        pressure_pa: 655_733.4656652204,
                        expected_region: PhaseEquilibriumRegion::TwoPhase,
                        expected_bubble_temperature_k: 299.99998279487806,
                        expected_dew_temperature_k: 301.05706034196055,
                    },
                    ThermoPressureBoundaryPerturbationCase {
                        label: "z=[0.23, 0.77] bubble-boundary + 0.1 Pa",
                        pressure_pa: 655_733.6656652204,
                        expected_region: PhaseEquilibriumRegion::LiquidOnly,
                        expected_bubble_temperature_k: 300.00001720512125,
                        expected_dew_temperature_k: 301.0570947339811,
                    },
                    ThermoPressureBoundaryPerturbationCase {
                        label: "z=[0.23, 0.77] dew-boundary + 0.1 Pa",
                        pressure_pa: 649_591.985824445,
                        expected_region: PhaseEquilibriumRegion::TwoPhase,
                        expected_bubble_temperature_k: 298.9423728489804,
                        expected_dew_temperature_k: 300.0000172272056,
                    },
                    ThermoPressureBoundaryPerturbationCase {
                        label: "z=[0.23, 0.77] dew-boundary - 0.1 Pa",
                        pressure_pa: 649_591.7858244451,
                        expected_region: PhaseEquilibriumRegion::VaporOnly,
                        expected_bubble_temperature_k: 298.9423383758716,
                        expected_dew_temperature_k: 299.9999827727937,
                    },
                ],
                temperature_cases: vec![
                    ThermoTemperatureBoundaryPerturbationCase {
                        label: "z=[0.23, 0.77] bubble-temperature - 0.001 K",
                        temperature_k: 299.011696117503,
                        expected_region: PhaseEquilibriumRegion::LiquidOnly,
                        expected_bubble_pressure_pa: 649_994.1976695253,
                        expected_dew_pressure_pa: 643_859.9583286671,
                    },
                    ThermoTemperatureBoundaryPerturbationCase {
                        label: "z=[0.23, 0.77] bubble-temperature + 0.001 K",
                        temperature_k: 299.01369611750295,
                        expected_region: PhaseEquilibriumRegion::TwoPhase,
                        expected_bubble_pressure_pa: 650_005.8023405897,
                        expected_dew_pressure_pa: 643_871.5477872168,
                    },
                    ThermoTemperatureBoundaryPerturbationCase {
                        label: "z=[0.23, 0.77] dew-temperature - 0.001 K",
                        temperature_k: 300.0693023846019,
                        expected_region: PhaseEquilibriumRegion::TwoPhase,
                        expected_bubble_pressure_pa: 656_136.3903625292,
                        expected_dew_pressure_pa: 649_994.1945257499,
                    },
                    ThermoTemperatureBoundaryPerturbationCase {
                        label: "z=[0.23, 0.77] dew-temperature + 0.001 K",
                        temperature_k: 300.07130238460184,
                        expected_region: PhaseEquilibriumRegion::VaporOnly,
                        expected_bubble_pressure_pa: 656_148.016201093,
                        expected_dew_pressure_pa: 650_005.8054843098,
                    },
                ],
            },
        ]
    }

    #[test]
    fn antoine_coefficients_estimate_saturation_pressure_in_pascal() {
        let coefficients = AntoineCoefficients::new(5.0, 1_200.0, 0.0);
        let pressure = coefficients
            .saturation_pressure_pa(300.0)
            .expect("expected saturation pressure");

        assert_close(pressure, std::f64::consts::E * 1_000.0, 1e-9);
    }

    #[test]
    fn antoine_coefficients_estimate_saturation_temperature_in_kelvin() {
        let coefficients = AntoineCoefficients::new(5.0, 1_200.0, 0.0);
        let temperature = coefficients
            .saturation_temperature_k(std::f64::consts::E * 1_000.0)
            .expect("expected saturation temperature");

        assert_close(temperature, 300.0, 1e-9);
    }

    #[test]
    fn placeholder_provider_estimates_ideal_k_values_from_saturation_pressure() {
        let mut methane = ThermoComponent::new(ComponentId::new("methane"), "Methane");
        methane.antoine = Some(AntoineCoefficients::new(50.0_f64.ln(), 0.0, 0.0));

        let mut ethane = ThermoComponent::new(ComponentId::new("ethane"), "Ethane");
        ethane.antoine = Some(AntoineCoefficients::new(25.0_f64.ln(), 0.0, 0.0));

        let provider = PlaceholderThermoProvider::new(ThermoSystem::binary([methane, ethane]));
        let state = ThermoState::new(300.0, 50_000.0, vec![0.4, 0.6]);

        let k_values = provider
            .estimate_k_values(&state)
            .expect("expected K-value estimation");

        assert_eq!(k_values.len(), 2);
        assert_close(k_values[0], 1.0, 1e-12);
        assert_close(k_values[1], 0.5, 1e-12);
    }

    #[test]
    fn placeholder_provider_estimates_phase_molar_enthalpy_from_heat_capacity() {
        let mut methane = ThermoComponent::new(ComponentId::new("methane"), "Methane");
        methane.liquid_heat_capacity_j_per_mol_k = Some(35.0);
        methane.vapor_heat_capacity_j_per_mol_k = Some(36.5);

        let mut ethane = ThermoComponent::new(ComponentId::new("ethane"), "Ethane");
        ethane.liquid_heat_capacity_j_per_mol_k = Some(52.0);
        ethane.vapor_heat_capacity_j_per_mol_k = Some(65.0);

        let provider = PlaceholderThermoProvider::new(ThermoSystem::binary([methane, ethane]));
        let liquid_state =
            PhaseThermoState::new(PhaseLabel::Liquid, 300.0, 650_000.0, vec![0.2, 0.8]);
        let vapor_state =
            PhaseThermoState::new(PhaseLabel::Vapor, 300.0, 650_000.0, vec![0.2, 0.8]);

        let liquid_enthalpy = provider
            .phase_molar_enthalpy(&liquid_state)
            .expect("expected liquid enthalpy");
        let vapor_enthalpy = provider
            .phase_molar_enthalpy(&vapor_state)
            .expect("expected vapor enthalpy");

        assert_close(liquid_enthalpy, 89.91, 1e-10);
        assert_close(vapor_enthalpy, 109.705, 1e-10);
    }

    #[test]
    fn placeholder_provider_estimates_bubble_and_dew_pressures() {
        let mut methane = ThermoComponent::new(ComponentId::new("methane"), "Methane");
        methane.antoine = Some(AntoineCoefficients::new(50.0_f64.ln(), 0.0, 0.0));

        let mut ethane = ThermoComponent::new(ComponentId::new("ethane"), "Ethane");
        ethane.antoine = Some(AntoineCoefficients::new(25.0_f64.ln(), 0.0, 0.0));

        let provider = PlaceholderThermoProvider::new(ThermoSystem::binary([methane, ethane]));
        let pressures = provider
            .estimate_bubble_dew_pressures(&BubbleDewPressureInput::new(300.0, vec![0.4, 0.6]))
            .expect("expected bubble/dew pressures");

        assert_close(pressures.bubble_pressure_pa, 35_000.0, 1e-9);
        assert_close(pressures.dew_pressure_pa, 31_250.0, 1e-9);
    }

    #[test]
    fn placeholder_provider_estimates_bubble_and_dew_temperatures() {
        let reference_pressure_pa = 100_000.0;
        let reference_temperature_k = 300.0;
        let boundary_slope = 250.0;

        let mut methane = ThermoComponent::new(ComponentId::new("methane"), "Methane");
        methane.antoine = Some(AntoineCoefficients::new(
            ((2.0_f64 * reference_pressure_pa) / 1_000.0_f64).ln()
                + boundary_slope / reference_temperature_k,
            boundary_slope,
            0.0,
        ));

        let mut ethane = ThermoComponent::new(ComponentId::new("ethane"), "Ethane");
        ethane.antoine = Some(AntoineCoefficients::new(
            ((0.5_f64 * reference_pressure_pa) / 1_000.0_f64).ln()
                + boundary_slope / reference_temperature_k,
            boundary_slope,
            0.0,
        ));

        let provider = PlaceholderThermoProvider::new(ThermoSystem::binary([methane, ethane]));
        let temperatures = provider
            .estimate_bubble_dew_temperatures(&BubbleDewTemperatureInput::new(
                reference_pressure_pa,
                vec![0.5, 0.5],
            ))
            .expect("expected bubble/dew temperatures");

        assert_close(temperatures.bubble_temperature_k, 236.635560732978, 1e-4);
        assert_close(temperatures.dew_temperature_k, 409.708580367858, 1e-4);
    }

    #[test]
    fn binary_hydrocarbon_lite_two_phase_pressure_boundary_perturbations_keep_window_consistent() {
        let provider = build_binary_hydrocarbon_lite_provider();
        for scenario in binary_hydrocarbon_lite_two_phase_boundary_scenarios() {
            assert_pressure_boundary_perturbations_keep_window_consistent(
                &provider,
                &scenario.overall_mole_fractions,
                300.0,
                scenario.exact_bubble_pressure_pa,
                scenario.exact_dew_pressure_pa,
                &scenario.pressure_cases,
            );
        }
    }

    #[test]
    fn binary_hydrocarbon_lite_two_phase_temperature_boundary_perturbations_keep_window_consistent()
    {
        let provider = build_binary_hydrocarbon_lite_provider();
        for scenario in binary_hydrocarbon_lite_two_phase_boundary_scenarios() {
            assert_temperature_boundary_perturbations_keep_window_consistent(
                &provider,
                &scenario.overall_mole_fractions,
                650_000.0,
                scenario.exact_bubble_temperature_k,
                scenario.exact_dew_temperature_k,
                &scenario.temperature_cases,
            );
        }
    }

    #[test]
    fn synthetic_pressure_boundary_perturbations_keep_window_consistent() {
        let liquid_only_provider = build_synthetic_provider([0.8, 0.6], 100_000.0);
        let liquid_only_cases = [
            ThermoPressureBoundaryPerturbationCase {
                label: "synthetic liquid-only bubble-boundary - 0.1 Pa",
                pressure_pa: 64_999.89999999998,
                expected_region: PhaseEquilibriumRegion::TwoPhase,
                expected_bubble_temperature_k: 299.9994461564153,
                expected_dew_temperature_k: 305.686744831391,
            },
            ThermoPressureBoundaryPerturbationCase {
                label: "synthetic liquid-only bubble-boundary + 0.1 Pa",
                pressure_pa: 65_000.09999999998,
                expected_region: PhaseEquilibriumRegion::LiquidOnly,
                expected_bubble_temperature_k: 300.0005538466912,
                expected_dew_temperature_k: 305.68789492268525,
            },
            ThermoPressureBoundaryPerturbationCase {
                label: "synthetic liquid-only dew-boundary + 0.1 Pa",
                pressure_pa: 64_000.099999999984,
                expected_region: PhaseEquilibriumRegion::TwoPhase,
                expected_bubble_temperature_k: 294.52098232922526,
                expected_dew_temperature_k: 300.00056250327293,
            },
            ThermoPressureBoundaryPerturbationCase {
                label: "synthetic liquid-only dew-boundary - 0.1 Pa",
                pressure_pa: 63_999.89999999999,
                expected_region: PhaseEquilibriumRegion::VaporOnly,
                expected_bubble_temperature_k: 294.51989804717425,
                expected_dew_temperature_k: 299.9994375027239,
            },
        ];
        assert_pressure_boundary_perturbations_keep_window_consistent(
            &liquid_only_provider,
            &[0.25, 0.75],
            300.0,
            64_999.99999999998,
            63_999.999999999985,
            &liquid_only_cases,
        );

        let vapor_only_provider = build_synthetic_provider([1.8, 1.3], 100_000.0);
        let vapor_only_cases = [
            ThermoPressureBoundaryPerturbationCase {
                label: "synthetic vapor-only bubble-boundary - 0.1 Pa",
                pressure_pa: 142_499.89999999997,
                expected_region: PhaseEquilibriumRegion::TwoPhase,
                expected_bubble_temperature_k: 299.99974736774993,
                expected_dew_temperature_k: 307.3140804747562,
            },
            ThermoPressureBoundaryPerturbationCase {
                label: "synthetic vapor-only bubble-boundary + 0.1 Pa",
                pressure_pa: 142_500.09999999998,
                expected_region: PhaseEquilibriumRegion::LiquidOnly,
                expected_bubble_temperature_k: 300.00025263320515,
                expected_dew_temperature_k: 307.3146106783237,
            },
            ThermoPressureBoundaryPerturbationCase {
                label: "synthetic vapor-only dew-boundary + 0.1 Pa",
                pressure_pa: 139_701.5925373134,
                expected_region: PhaseEquilibriumRegion::TwoPhase,
                expected_bubble_temperature_k: 293.0259814729438,
                expected_dew_temperature_k: 300.0002576946889,
            },
            ThermoPressureBoundaryPerturbationCase {
                label: "synthetic vapor-only dew-boundary - 0.1 Pa",
                pressure_pa: 139_701.3925373134,
                expected_region: PhaseEquilibriumRegion::VaporOnly,
                expected_bubble_temperature_k: 293.02548977289626,
                expected_dew_temperature_k: 299.9997423083373,
            },
        ];
        assert_pressure_boundary_perturbations_keep_window_consistent(
            &vapor_only_provider,
            &[0.25, 0.75],
            300.0,
            142_499.99999999997,
            139_701.4925373134,
            &vapor_only_cases,
        );
    }

    #[test]
    fn synthetic_temperature_boundary_perturbations_keep_window_consistent() {
        let liquid_only_provider = build_synthetic_provider([0.8, 0.6], 100_000.0);
        let liquid_only_cases = [
            ThermoTemperatureBoundaryPerturbationCase {
                label: "synthetic liquid-only bubble-temperature - 0.001 K",
                temperature_k: 621.0392207837401,
                expected_region: PhaseEquilibriumRegion::LiquidOnly,
                expected_bubble_pressure_pa: 99_999.93518115903,
                expected_dew_pressure_pa: 98_461.47463991042,
            },
            ThermoTemperatureBoundaryPerturbationCase {
                label: "synthetic liquid-only bubble-temperature + 0.001 K",
                temperature_k: 621.0412207837401,
                expected_region: PhaseEquilibriumRegion::TwoPhase,
                expected_bubble_pressure_pa: 100_000.06481862975,
                expected_dew_pressure_pa: 98_461.60228295853,
            },
            ThermoTemperatureBoundaryPerturbationCase {
                label: "synthetic liquid-only dew-temperature - 0.001 K",
                temperature_k: 645.9166712270983,
                expected_region: PhaseEquilibriumRegion::TwoPhase,
                expected_bubble_pressure_pa: 101_562.43914080938,
                expected_dew_pressure_pa: 99_999.94007710462,
            },
            ThermoTemperatureBoundaryPerturbationCase {
                label: "synthetic liquid-only dew-temperature + 0.001 K",
                temperature_k: 645.9186712270982,
                expected_region: PhaseEquilibriumRegion::VaporOnly,
                expected_bubble_pressure_pa: 101_562.560857197,
                expected_dew_pressure_pa: 100_000.05992093244,
            },
        ];
        assert_temperature_boundary_perturbations_keep_window_consistent(
            &liquid_only_provider,
            &[0.25, 0.75],
            100_000.0,
            621.0402207837401,
            645.9176712270983,
            &liquid_only_cases,
        );

        let vapor_only_provider = build_synthetic_provider([1.8, 1.3], 100_000.0);
        let vapor_only_cases = [
            ThermoTemperatureBoundaryPerturbationCase {
                label: "synthetic vapor-only bubble-temperature - 0.001 K",
                temperature_k: 210.52440329728063,
                expected_region: PhaseEquilibriumRegion::LiquidOnly,
                expected_bubble_pressure_pa: 99_999.43593205792,
                expected_dew_pressure_pa: 98_035.58212349434,
            },
            ThermoTemperatureBoundaryPerturbationCase {
                label: "synthetic vapor-only bubble-temperature + 0.001 K",
                temperature_k: 210.52640329728064,
                expected_region: PhaseEquilibriumRegion::TwoPhase,
                expected_bubble_pressure_pa: 100_000.56406683738,
                expected_dew_pressure_pa: 98_036.68810323098,
            },
            ThermoTemperatureBoundaryPerturbationCase {
                label: "synthetic vapor-only dew-temperature - 0.001 K",
                temperature_k: 214.10038569784498,
                expected_region: PhaseEquilibriumRegion::TwoPhase,
                expected_bubble_pressure_pa: 102_002.64881882841,
                expected_dew_pressure_pa: 99_999.45461578779,
            },
            ThermoTemperatureBoundaryPerturbationCase {
                label: "synthetic vapor-only dew-temperature + 0.001 K",
                temperature_k: 214.102385697845,
                expected_region: PhaseEquilibriumRegion::VaporOnly,
                expected_bubble_pressure_pa: 102_003.76143371491,
                expected_dew_pressure_pa: 100_000.54538042123,
            },
        ];
        assert_temperature_boundary_perturbations_keep_window_consistent(
            &vapor_only_provider,
            &[0.25, 0.75],
            100_000.0,
            210.52540329728063,
            214.101385697845,
            &vapor_only_cases,
        );
    }

    #[test]
    fn placeholder_provider_rejects_missing_phase_heat_capacity() {
        let methane = ThermoComponent::new(ComponentId::new("methane"), "Methane");
        let provider = PlaceholderThermoProvider::new(ThermoSystem::new(vec![methane]));
        let state = PhaseThermoState::new(PhaseLabel::Liquid, 300.0, 101_325.0, vec![1.0]);

        let error = provider
            .phase_molar_enthalpy(&state)
            .expect_err("expected missing heat capacity");

        assert_eq!(error.code().as_str(), "thermo");
        assert!(error.message().contains("missing `liquid` heat capacity"));
    }

    #[test]
    fn placeholder_provider_rejects_unnormalized_mole_fractions() {
        let mut methane = ThermoComponent::new(ComponentId::new("methane"), "Methane");
        methane.antoine = Some(AntoineCoefficients::new(50.0_f64.ln(), 0.0, 0.0));

        let mut ethane = ThermoComponent::new(ComponentId::new("ethane"), "Ethane");
        ethane.antoine = Some(AntoineCoefficients::new(25.0_f64.ln(), 0.0, 0.0));

        let provider = PlaceholderThermoProvider::new(ThermoSystem::binary([methane, ethane]));
        let state = ThermoState::new(300.0, 50_000.0, vec![0.4, 0.4]);

        let error = provider
            .estimate_k_values(&state)
            .expect_err("expected unnormalized mole fractions to be rejected");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(error.message().contains("must sum to one"));
    }

    #[test]
    fn placeholder_provider_rejects_unnormalized_bubble_dew_input() {
        let mut methane = ThermoComponent::new(ComponentId::new("methane"), "Methane");
        methane.antoine = Some(AntoineCoefficients::new(50.0_f64.ln(), 0.0, 0.0));

        let mut ethane = ThermoComponent::new(ComponentId::new("ethane"), "Ethane");
        ethane.antoine = Some(AntoineCoefficients::new(25.0_f64.ln(), 0.0, 0.0));

        let provider = PlaceholderThermoProvider::new(ThermoSystem::binary([methane, ethane]));
        let error = provider
            .estimate_bubble_dew_pressures(&BubbleDewPressureInput::new(300.0, vec![0.4, 0.4]))
            .expect_err("expected unnormalized mole fractions to be rejected");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(error.message().contains("must sum to one"));
    }

    #[test]
    fn placeholder_provider_rejects_unnormalized_bubble_dew_temperature_input() {
        let reference_pressure_pa = 100_000.0;
        let boundary_slope = 250.0;

        let mut methane = ThermoComponent::new(ComponentId::new("methane"), "Methane");
        methane.antoine = Some(AntoineCoefficients::new(
            ((2.0_f64 * reference_pressure_pa) / 1_000.0_f64).ln() + boundary_slope / 300.0,
            boundary_slope,
            0.0,
        ));

        let mut ethane = ThermoComponent::new(ComponentId::new("ethane"), "Ethane");
        ethane.antoine = Some(AntoineCoefficients::new(
            ((0.5_f64 * reference_pressure_pa) / 1_000.0_f64).ln() + boundary_slope / 300.0,
            boundary_slope,
            0.0,
        ));

        let provider = PlaceholderThermoProvider::new(ThermoSystem::binary([methane, ethane]));
        let error = provider
            .estimate_bubble_dew_temperatures(&BubbleDewTemperatureInput::new(
                reference_pressure_pa,
                vec![0.4, 0.4],
            ))
            .expect_err("expected unnormalized mole fractions to be rejected");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(error.message().contains("must sum to one"));
    }

    #[test]
    fn placeholder_provider_rejects_missing_antoine_coefficients() {
        let methane = ThermoComponent::new(ComponentId::new("methane"), "Methane");
        let provider = PlaceholderThermoProvider::new(ThermoSystem::new(vec![methane]));
        let state = ThermoState::new(300.0, 101_325.0, vec![1.0]);

        let error = provider
            .estimate_k_values(&state)
            .expect_err("expected missing Antoine coefficients");

        assert_eq!(error.code().as_str(), "thermo");
        assert!(error.message().contains("missing Antoine coefficients"));
    }

    #[test]
    fn package_provider_returns_manifest_and_system_for_known_package() {
        let methane_id = ComponentId::new("methane");
        let system = ThermoSystem::new(vec![ThermoComponent::new(methane_id.clone(), "Methane")]);
        let manifest = PropertyPackageManifest::new(
            "methane-basic-v1",
            "2026.03.1",
            PropertyPackageSource::RemoteDerivedPackage,
            vec![methane_id],
        );
        let provider = InMemoryPropertyPackageProvider::new(vec![(manifest, system.clone())]);

        let manifests = provider.list_manifests();
        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].package_id, "methane-basic-v1");

        let loaded = provider
            .load_system("methane-basic-v1")
            .expect("expected thermo system");
        assert_eq!(loaded, system);
    }

    #[test]
    fn package_provider_reports_missing_package() {
        let provider = InMemoryPropertyPackageProvider::default();
        let error = provider
            .load_system("missing-package")
            .expect_err("expected missing package error");

        assert_eq!(error.code().as_str(), "missing_entity");
    }

    #[test]
    fn remote_evaluation_manifest_defaults_to_remote_only() {
        let manifest = PropertyPackageManifest::new(
            "remote-eval-pkg",
            "2026.03.1",
            PropertyPackageSource::RemoteEvaluationService,
            vec![],
        );

        assert_eq!(
            manifest.classification,
            PropertyPackageClassification::RemoteOnly
        );
    }

    #[test]
    fn cached_provider_loads_local_packages_from_store_cache_layout() {
        let root = unique_temp_path("cached-provider");
        let mut index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-credential"),
        );
        let mut record = StoredPropertyPackageRecord::new(
            "methane-basic-v1",
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            "",
            0,
            timestamp(100),
        );
        record.expires_at = Some(timestamp(800));
        let payload = StoredPropertyPackagePayload::new(
            "methane-basic-v1",
            "2026.03.1",
            vec![StoredThermoComponent::new(
                ComponentId::new("methane"),
                "Methane",
            )],
        );
        let integrity =
            property_package_payload_integrity(&payload).expect("expected payload integrity");
        let manifest = {
            let mut manifest = StoredPropertyPackageManifest::new(
                "methane-basic-v1",
                "2026.03.1",
                StoredPropertyPackageSource::RemoteDerivedPackage,
                vec![ComponentId::new("methane")],
            );
            manifest.hash = integrity.hash.clone();
            manifest.size_bytes = integrity.size_bytes;
            manifest.expires_at = Some(timestamp(800));
            manifest
        };
        record.hash = integrity.hash.clone();
        record.size_bytes = integrity.size_bytes;

        write_property_package_manifest(record.manifest_path_under(&root), &manifest)
            .expect("expected manifest write");
        write_property_package_payload(
            record
                .payload_path_under(&root)
                .expect("expected payload path"),
            &payload,
        )
        .expect("expected payload write");
        index.property_packages.push(record);

        let provider = CachedPropertyPackageProvider::new_at(&root, &index, timestamp(700))
            .expect("expected cached provider");
        let manifests = provider.list_manifests();

        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].package_id, "methane-basic-v1");
        assert_eq!(manifests[0].hash, integrity.hash);
        let system = provider
            .load_system("methane-basic-v1")
            .expect("expected thermo system");
        assert_eq!(system.component_count(), 1);
        assert_eq!(system.components[0].name, "Methane");

        fs::remove_dir_all(&root).expect("expected temp dir cleanup");
    }

    #[test]
    fn cached_provider_skips_expired_records_before_touching_disk() {
        let root = unique_temp_path("cached-provider-expired");
        let mut index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-credential"),
        );
        let mut record = StoredPropertyPackageRecord::new(
            "expired-pkg",
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            "sha256:expired",
            256,
            timestamp(100),
        );
        record.expires_at = Some(timestamp(200));
        index.property_packages.push(record);

        let provider = CachedPropertyPackageProvider::new_at(&root, &index, timestamp(300))
            .expect("expected cached provider");

        assert!(provider.list_manifests().is_empty());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn cached_provider_rejects_manifest_mismatch_against_index_record() {
        let root = unique_temp_path("cached-provider-mismatch");
        let mut index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-credential"),
        );
        let mut record = StoredPropertyPackageRecord::new(
            "pkg-1",
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            "sha256:expected",
            0,
            timestamp(100),
        );
        record.expires_at = Some(timestamp(500));
        let mut manifest = StoredPropertyPackageManifest::new(
            "pkg-1",
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            vec![ComponentId::new("methane")],
        );
        let payload = StoredPropertyPackagePayload::new(
            "pkg-1",
            "2026.03.1",
            vec![StoredThermoComponent::new(
                ComponentId::new("methane"),
                "Methane",
            )],
        );
        let integrity =
            property_package_payload_integrity(&payload).expect("expected payload integrity");
        manifest.hash = integrity.hash;
        manifest.size_bytes = integrity.size_bytes;
        manifest.expires_at = Some(timestamp(500));
        record.size_bytes = integrity.size_bytes;

        write_property_package_manifest(record.manifest_path_under(&root), &manifest)
            .expect("expected manifest write");
        write_property_package_payload(
            record
                .payload_path_under(&root)
                .expect("expected payload path"),
            &payload,
        )
        .expect("expected payload write");
        index.property_packages.push(record);

        let error = CachedPropertyPackageProvider::new_at(&root, &index, timestamp(300))
            .expect_err("expected manifest mismatch");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(error.message().contains("does not match manifest hash"));
        fs::remove_dir_all(&root).expect("expected temp dir cleanup");
    }

    #[test]
    fn cached_provider_loads_example_property_package_files() {
        let root = unique_temp_path("cached-provider-example");
        let example_manifest_path = example_package_path("manifest.json");
        let example_payload_path = example_package_path("payload.rfpkg");
        let manifest =
            read_property_package_manifest(&example_manifest_path).expect("expected manifest read");
        let payload =
            read_property_package_payload(&example_payload_path).expect("expected payload read");

        let mut index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-credential"),
        );
        let mut record = StoredPropertyPackageRecord::new(
            &manifest.package_id,
            &manifest.version,
            manifest.source,
            manifest.hash.clone(),
            manifest.size_bytes,
            timestamp(100),
        );
        record.expires_at = manifest.expires_at;

        write_property_package_manifest(record.manifest_path_under(&root), &manifest)
            .expect("expected manifest write");
        write_property_package_payload(
            record
                .payload_path_under(&root)
                .expect("expected payload path"),
            &payload,
        )
        .expect("expected payload write");
        index.property_packages.push(record);

        let provider = CachedPropertyPackageProvider::new_at(&root, &index, timestamp(150))
            .expect("expected cached provider");
        let system = provider
            .load_system("binary-hydrocarbon-lite-v1")
            .expect("expected thermo system");

        assert_eq!(system.component_count(), 2);
        assert_eq!(system.components[0].id.as_str(), "methane");
        assert_eq!(system.components[1].id.as_str(), "ethane");

        fs::remove_dir_all(&root).expect("expected temp dir cleanup");
    }

    fn timestamp(seconds: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn unique_temp_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected time after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("radishflow-{name}-{unique}"))
    }

    fn example_package_path(file_name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/sample-components/property-packages/binary-hydrocarbon-lite-v1")
            .join(file_name)
    }
}
