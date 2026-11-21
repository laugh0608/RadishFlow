//! v4 project envelope: explicit flow capabilities and a persisted active plan.
//! Legacy project entry points retain their existing capability rejection.
mod document;
mod equation_oriented;
pub use equation_oriented::*;
mod file_io;
mod mixed;
mod object_identities;
mod plan;
mod simultaneous;
mod single_phase;
use crate::project::STORED_PROJECT_FILE_KIND;
pub use crate::project_v3::EquipmentSupport;
use crate::project_v3::{
    StoredEngineeringInputs,
    shared::{self, EngineeringRead, PresentationWire},
};
use crate::{StoredDocumentMetadata, StoredProjectDocument};
pub use file_io::*;
pub use mixed::*;
use object_identities::ObjectIdentitiesWire;
pub use plan::*;
use rf_types::{FlowObjectIdentities, RfError, RfResult, units::DisplayUnitSet};
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub use simultaneous::*;
pub use single_phase::*;
use std::collections::BTreeSet;

pub const VERSION: u32 = 4;
pub const FLOW_PLAN_CAPABILITY: &str = "radishflow.flow-plan";
pub const SPLITTER_CAPABILITY: &str = "radishflow.unit.splitter";
pub const PARTITIONED_CAPABILITY: &str = "radishflow.solver.partitioned-sm";
pub const MIXED_CAPABILITY: &str = "radishflow.solver.fixed-tp-mixed-material";
pub const MIXED_RECYCLE_CAPABILITY: &str = "radishflow.solver.fixed-tp-mixed-recycle";
pub const EQUATION_ORIENTED_CAPABILITY: &str = "radishflow.solver.fixed-tp-material-eo";
pub const SIMULTANEOUS_CAPABILITY: &str = "radishflow.solver.fixed-tp-affine-modular-lu";
pub const OBJECT_IDENTITY_CAPABILITY: &str = "radishflow.object-identity";
pub const VARIABLE_WATCH_LIST_CAPABILITY: &str = "radishflow.variable-watch-list";
pub const CONCENTRIC_TUBE_RATING_CAPABILITY: &str = "radishflow.equipment.concentric-tube-rating";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectIdentitySupport {
    Supported,
    Unavailable,
}

/// A consumer's supported vocabulary is independent of the project envelope version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FlowPlanSupport {
    Sequential,
    Simultaneous,
    EquationOriented,
    Mixed,
    MixedRecycle,
    ProductComposition,
    SinglePhase,
    SinglePhaseRecycle,
}

impl FlowPlanSupport {
    fn supports_method(self, method: &str) -> bool {
        match method {
            ACYCLIC_METHOD | PARTITIONED_METHOD => true,
            SIMULTANEOUS_METHOD => self >= Self::Simultaneous,
            EQUATION_ORIENTED_METHOD => self >= Self::EquationOriented,
            MIXED_METHOD => self >= Self::Mixed,
            MIXED_RECYCLE_METHOD => self >= Self::MixedRecycle,
            SINGLE_PHASE_METHOD => self >= Self::SinglePhase,
            SINGLE_PHASE_RECYCLE_METHOD => self >= Self::SinglePhaseRecycle,
            _ => false,
        }
    }

    pub fn supports_equation_version(self, version: u32) -> bool {
        match version {
            1 => self >= Self::EquationOriented,
            PRODUCT_COMPOSITION_VERSION => self >= Self::ProductComposition,
            _ => false,
        }
    }

    fn supports_capability(
        self,
        capability: &RequiredCapability,
        identities: ObjectIdentitySupport,
        equipment: EquipmentSupport,
    ) -> bool {
        if capability.id == EQUATION_ORIENTED_CAPABILITY {
            return self.supports_equation_version(capability.version);
        }
        if capability.version != 1 {
            return false;
        }
        match capability.id.as_str() {
            FLOW_PLAN_CAPABILITY | SPLITTER_CAPABILITY | PARTITIONED_CAPABILITY => true,
            SIMULTANEOUS_CAPABILITY => self.supports_method(SIMULTANEOUS_METHOD),
            MIXED_CAPABILITY => self.supports_method(MIXED_METHOD),
            MIXED_RECYCLE_CAPABILITY => self.supports_method(MIXED_RECYCLE_METHOD),
            SINGLE_PHASE_CAPABILITY => self.supports_method(SINGLE_PHASE_METHOD),
            SINGLE_PHASE_RECYCLE_CAPABILITY => self.supports_method(SINGLE_PHASE_RECYCLE_METHOD),
            OBJECT_IDENTITY_CAPABILITY | VARIABLE_WATCH_LIST_CAPABILITY => {
                identities == ObjectIdentitySupport::Supported
            }
            CONCENTRIC_TUBE_RATING_CAPABILITY => {
                equipment == EquipmentSupport::ConcentricTubeRating
            }
            _ => false,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredCapability {
    pub id: String,
    pub version: u32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct StoredProjectV4 {
    pub document: StoredProjectDocument,
    pub display_units: DisplayUnitSet,
    pub engineering: Option<StoredEngineeringInputs>,
    pub steady_state_plan: StoredFlowPlan,
    pub object_identities: Option<FlowObjectIdentities>,
    pub variable_watch_list: Option<Vec<String>>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectV4Read {
    Editable(Box<StoredProjectV4>),
    ReadOnly(ReadOnlyProjectV4),
}
#[derive(Debug, Clone, PartialEq)]
pub struct ReadOnlyProjectV4 {
    original_json: String,
    metadata: StoredDocumentMetadata,
    revision: u64,
    reasons: Vec<String>,
}
impl ReadOnlyProjectV4 {
    pub fn original_json(&self) -> &str {
        &self.original_json
    }
    pub fn metadata(&self) -> &StoredDocumentMetadata {
        &self.metadata
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn reasons(&self) -> &[String] {
        &self.reasons
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadWire {
    kind: String,
    schema_version: u32,
    required_capabilities: Vec<RequiredCapability>,
    document: Value,
    presentation: Value,
    simulation: Value,
    #[serde(default)]
    engineering: Option<serde::de::IgnoredAny>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WriteWire<'a> {
    kind: &'static str,
    schema_version: u32,
    required_capabilities: Vec<RequiredCapability>,
    document: &'a StoredProjectDocument,
    presentation: PresentationWire,
    simulation: Simulation,
    #[serde(skip_serializing_if = "Option::is_none")]
    engineering: Option<shared::EngineeringSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    object_identities: Option<ObjectIdentitiesWire>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variable_watch_list: Option<&'a [String]>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Simulation {
    steady_state_plan: StoredFlowPlan,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SimulationProbe {
    steady_state_plan: PlanProbe,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlanProbe {
    method: StoredMethod,
    acceptance_contract_version: Option<u32>,
    response_contract_version: Option<u32>,
    equation_contract_version: Option<u32>,
    blocks: Option<Vec<Value>>,
}

pub fn required_capabilities(project: &StoredProjectV4) -> Vec<RequiredCapability> {
    let mut ids = vec![(FLOW_PLAN_CAPABILITY, 1)];
    if project.object_identities.is_some() {
        ids.push((OBJECT_IDENTITY_CAPABILITY, 1));
    }
    if project.variable_watch_list.is_some() {
        ids.push((VARIABLE_WATCH_LIST_CAPABILITY, 1));
    }
    if project
        .engineering
        .as_ref()
        .is_some_and(StoredEngineeringInputs::has_concentric_tube_rating)
    {
        ids.push((CONCENTRIC_TUBE_RATING_CAPABILITY, 1));
    }
    if project
        .document
        .flowsheet
        .units
        .values()
        .any(|u| u.kind == "splitter")
    {
        ids.push((SPLITTER_CAPABILITY, 1));
    }
    if matches!(project.steady_state_plan, StoredFlowPlan::Partitioned(_)) {
        ids.push((PARTITIONED_CAPABILITY, 1));
    }
    if matches!(project.steady_state_plan, StoredFlowPlan::Simultaneous(_)) {
        ids.push((SIMULTANEOUS_CAPABILITY, 1));
    }
    if let StoredFlowPlan::EquationOriented(o) = &project.steady_state_plan {
        ids.push((EQUATION_ORIENTED_CAPABILITY, o.contract_version()));
    }
    if matches!(project.steady_state_plan, StoredFlowPlan::Mixed(_)) {
        ids.push((MIXED_CAPABILITY, 1));
    }
    if matches!(project.steady_state_plan, StoredFlowPlan::MixedRecycle(_)) {
        ids.push((MIXED_RECYCLE_CAPABILITY, 1));
    }
    if matches!(project.steady_state_plan, StoredFlowPlan::SinglePhase(_)) {
        ids.push((SINGLE_PHASE_CAPABILITY, 1));
    }
    if matches!(
        project.steady_state_plan,
        StoredFlowPlan::SinglePhaseRecycle(_)
    ) {
        ids.push((SINGLE_PHASE_RECYCLE_CAPABILITY, 1));
    }
    ids.sort();
    ids.into_iter()
        .map(|(id, version)| RequiredCapability {
            id: id.into(),
            version,
        })
        .collect()
}
pub fn parse(contents: &str) -> RfResult<ProjectV4Read> {
    parse_with_support(contents, FlowPlanSupport::Mixed)
}

pub fn parse_with_support(contents: &str, support: FlowPlanSupport) -> RfResult<ProjectV4Read> {
    parse_for_consumer(contents, support, ObjectIdentitySupport::Supported)
}

pub fn parse_for_consumer(
    contents: &str,
    support: FlowPlanSupport,
    identity_support: ObjectIdentitySupport,
) -> RfResult<ProjectV4Read> {
    parse_with_consumer_support(
        contents,
        support,
        identity_support,
        EquipmentSupport::SeparatorOnly,
    )
}

pub fn parse_with_equipment_support(
    contents: &str,
    support: FlowPlanSupport,
    equipment_support: EquipmentSupport,
) -> RfResult<ProjectV4Read> {
    parse_with_consumer_support(
        contents,
        support,
        ObjectIdentitySupport::Supported,
        equipment_support,
    )
}

fn parse_with_consumer_support(
    contents: &str,
    support: FlowPlanSupport,
    identity_support: ObjectIdentitySupport,
    equipment_support: EquipmentSupport,
) -> RfResult<ProjectV4Read> {
    crate::strict_json::validate(contents.as_bytes()).map_err(json_error)?;
    let wire: ReadWire = serde_json::from_str(contents).map_err(json_error)?;
    if wire.kind != STORED_PROJECT_FILE_KIND || wire.schema_version != VERSION {
        return Err(RfError::invalid_input(
            "project v4: unsupported kind or schemaVersion",
        ));
    }
    let mut seen = BTreeSet::new();
    for cap in &wire.required_capabilities {
        if cap.id.trim().is_empty() || cap.version == 0 || !seen.insert(&cap.id) {
            return Err(RfError::invalid_input(
                "requiredCapabilities: blank, duplicate or invalid capability",
            ));
        }
    }
    let mut reasons: Vec<String> = wire
        .required_capabilities
        .iter()
        .filter(|c| !support.supports_capability(c, identity_support, equipment_support))
        .map(|c| format!("{} / {}", c.id, c.version))
        .collect();
    if !reasons.is_empty() {
        return readonly(contents, &wire.document, reasons);
    }
    let probe: SimulationProbe =
        serde_json::from_value(wire.simulation.clone()).map_err(json_error)?;
    let modular = probe.steady_state_plan.method.id == SIMULTANEOUS_METHOD;
    let eo = probe.steady_state_plan.method.id == EQUATION_ORIENTED_METHOD;
    let mixed = matches!(
        probe.steady_state_plan.method.id.as_str(),
        MIXED_METHOD | MIXED_RECYCLE_METHOD
    );
    if (modular
        || eo
        || mixed
        || matches!(
            probe.steady_state_plan.method.id.as_str(),
            SINGLE_PHASE_METHOD | SINGLE_PHASE_RECYCLE_METHOD
        ))
        && (probe.steady_state_plan.method.version == 0
            || probe.steady_state_plan.acceptance_contract_version == Some(0)
            || probe.steady_state_plan.response_contract_version == Some(0)
            || probe.steady_state_plan.equation_contract_version == Some(0))
    {
        return Err(RfError::invalid_input(
            "simulation: contract versions must be positive",
        ));
    }
    if !support.supports_method(&probe.steady_state_plan.method.id)
        || probe.steady_state_plan.method.version != 1
        || (eo
            && probe
                .steady_state_plan
                .equation_contract_version
                .is_some_and(|v| !support.supports_equation_version(v)))
        || ((probe.steady_state_plan.method.id == PARTITIONED_METHOD || modular || eo || mixed)
            && probe
                .steady_state_plan
                .acceptance_contract_version
                .is_some_and(|v| {
                    if eo {
                        !support.supports_equation_version(v)
                    } else {
                        v != 1
                    }
                }))
        || (modular
            && probe
                .steady_state_plan
                .response_contract_version
                .is_some_and(|v| v != 1))
    {
        reasons.push("unsupported steady-state method or acceptance contract".into());
    }
    if mixed && let Some(blocks) = &probe.steady_state_plan.blocks {
        for block in blocks {
            match block.pointer("/method/kind").and_then(Value::as_str) {
                Some("local") => {}
                Some("equation_oriented") => {
                    let version = block
                        .pointer("/method/equationContractVersion")
                        .and_then(Value::as_u64)
                        .filter(|v| *v > 0 && *v <= u32::MAX as u64)
                        .ok_or_else(|| {
                            RfError::invalid_input(
                                "simulation: block equation contract must be a positive u32",
                            )
                        })?;
                    if version != 1 {
                        reasons.push(format!("unsupported block equation contract {version}"));
                    }
                }
                Some(kind) => reasons.push(format!("unsupported block method {kind}")),
                None => {
                    return Err(RfError::invalid_input(
                        "simulation: block method kind required",
                    ));
                }
            }
        }
    }
    if !reasons.is_empty() {
        return readonly(contents, &wire.document, reasons);
    }
    let engineering = match shared::decode(contents, equipment_support)? {
        EngineeringRead::Supported(inputs) => inputs,
        EngineeringRead::Unsupported(count) => {
            return readonly(
                contents,
                &wire.document,
                vec![format!("unsupported equipment capability ({count} tasks)")],
            );
        }
    };
    // Unknown capabilities own the entire envelope, including future root and
    // presentation fields. Only a fully supported project is decoded strictly.
    let root: Value = serde_json::from_str(contents).map_err(json_error)?;
    document::object(
        &root,
        &[
            "kind",
            "schemaVersion",
            "requiredCapabilities",
            "document",
            "presentation",
            "simulation",
            "engineering",
            "objectIdentities",
            "variableWatchList",
        ],
        "project",
    )?;
    let document = document::decode(&wire.document)?;
    let object_identities = root
        .get("objectIdentities")
        .map(|value| {
            serde_json::from_value::<ObjectIdentitiesWire>(value.clone())
                .map_err(json_error)?
                .decode(&document.flowsheet)
        })
        .transpose()?;
    let simulation: Simulation = serde_json::from_value(wire.simulation).map_err(json_error)?;
    let variable_watch_list = root
        .get("variableWatchList")
        .map(|value| serde_json::from_value::<Vec<String>>(value.clone()).map_err(json_error))
        .transpose()?;
    let project = StoredProjectV4 {
        document,
        display_units: serde_json::from_value::<PresentationWire>(wire.presentation)
            .map_err(json_error)?
            .into_display_units()?,
        engineering,
        steady_state_plan: simulation.steady_state_plan,
        object_identities,
        variable_watch_list,
    };
    validate_watch_list(&project)?;
    validate_plan_references(&project)?;
    let actual: BTreeSet<_> = wire.required_capabilities.into_iter().collect();
    if actual != required_capabilities(&project).into_iter().collect() {
        return Err(RfError::invalid_input(
            "requiredCapabilities: capability declarations do not match project content",
        ));
    }
    // The field is consumed by the shared section codec, which rejects explicit null.
    let _ = wire.engineering;
    Ok(ProjectV4Read::Editable(Box::new(project)))
}
fn readonly(contents: &str, value: &Value, reasons: Vec<String>) -> RfResult<ProjectV4Read> {
    #[derive(Deserialize)]
    struct Metadata {
        metadata: StoredDocumentMetadata,
        revision: u64,
    }
    let document: Metadata = serde_json::from_value(value.clone()).map_err(json_error)?;
    document.metadata.validate()?;
    Ok(ProjectV4Read::ReadOnly(ReadOnlyProjectV4 {
        original_json: contents.into(),
        metadata: document.metadata,
        revision: document.revision,
        reasons,
    }))
}
pub fn to_pretty_json(project: &ProjectV4Read) -> RfResult<String> {
    to_pretty_json_with_support(project, FlowPlanSupport::Mixed)
}

pub fn to_pretty_json_with_support(
    project: &ProjectV4Read,
    support: FlowPlanSupport,
) -> RfResult<String> {
    to_pretty_json_with_equipment_support(project, support, EquipmentSupport::SeparatorOnly)
}

pub fn to_pretty_json_with_equipment_support(
    project: &ProjectV4Read,
    support: FlowPlanSupport,
    equipment_support: EquipmentSupport,
) -> RfResult<String> {
    let ProjectV4Read::Editable(p) = project else {
        return Err(RfError::invalid_input(
            "project v4 is read-only; saving is prohibited",
        ));
    };
    if !support.supports_method(&p.steady_state_plan.method().id) {
        return Err(RfError::invalid_input(
            "project v4 writer does not support the selected steady-state method",
        )
        .with_diagnostic_code("store.project.unsupported_write_method"));
    }
    if let StoredFlowPlan::EquationOriented(o) = &p.steady_state_plan
        && !support.supports_equation_version(o.contract_version())
    {
        return Err(RfError::invalid_input(
            "consumer cannot write this EO contract version",
        ));
    }
    document::validate(&p.document)?;
    validate_watch_list(p)?;
    validate_plan_references(p)?;
    let wire = WriteWire {
        kind: STORED_PROJECT_FILE_KIND,
        schema_version: VERSION,
        required_capabilities: required_capabilities(p),
        document: &p.document,
        presentation: PresentationWire::from(&p.display_units),
        simulation: Simulation {
            steady_state_plan: p.steady_state_plan.clone(),
        },
        engineering: shared::encode(p.engineering.as_ref(), equipment_support)?,
        object_identities: p
            .object_identities
            .as_ref()
            .map(|identities| ObjectIdentitiesWire::encode(identities, &p.document.flowsheet))
            .transpose()?,
        variable_watch_list: p.variable_watch_list.as_deref(),
    };
    serde_json::to_string_pretty(&wire).map_err(json_error)
}

fn validate_plan_references(project: &StoredProjectV4) -> RfResult<()> {
    match &project.steady_state_plan {
        StoredFlowPlan::SinglePhase(plan) => plan.validate_references(&project.document.flowsheet),
        StoredFlowPlan::SinglePhaseRecycle(plan) => plan.validate_for(&project.document.flowsheet),
        _ => Ok(()),
    }
}

fn validate_watch_list(project: &StoredProjectV4) -> RfResult<()> {
    let Some(entries) = &project.variable_watch_list else {
        return Ok(());
    };
    if project.object_identities.is_none() {
        return Err(RfError::invalid_input(
            "variableWatchList requires objectIdentities",
        ));
    }
    let mut seen = BTreeSet::new();
    if entries
        .iter()
        .any(|entry| entry.trim().is_empty() || !seen.insert(entry))
    {
        return Err(RfError::invalid_input(
            "variableWatchList contains a blank or duplicate reference",
        ));
    }
    Ok(())
}
fn json_error(e: serde_json::Error) -> RfError {
    RfError::invalid_input(format!("project v4 JSON: {e}"))
}
#[cfg(test)]
mod tests;
