//! v4 project envelope: explicit flow capabilities and a persisted active plan.
//! Legacy project entry points retain their existing capability rejection.
mod document;
mod file_io;
mod plan;
use crate::project::STORED_PROJECT_FILE_KIND;
use crate::project_v3::{
    StoredEngineeringInputs,
    shared::{self, EngineeringRead, PresentationWire},
};
use crate::{StoredDocumentMetadata, StoredProjectDocument};
pub use file_io::*;
pub use plan::*;
use rf_types::{RfError, RfResult, units::DisplayUnitSet};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

pub const VERSION: u32 = 4;
pub const FLOW_PLAN_CAPABILITY: &str = "radishflow.flow-plan";
pub const SPLITTER_CAPABILITY: &str = "radishflow.unit.splitter";
pub const PARTITIONED_CAPABILITY: &str = "radishflow.solver.partitioned-sm";
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
}

pub fn required_capabilities(project: &StoredProjectV4) -> Vec<RequiredCapability> {
    let mut ids = vec![FLOW_PLAN_CAPABILITY];
    if project
        .document
        .flowsheet
        .units
        .values()
        .any(|u| u.kind == "splitter")
    {
        ids.push(SPLITTER_CAPABILITY);
    }
    if matches!(project.steady_state_plan, StoredFlowPlan::Partitioned(_)) {
        ids.push(PARTITIONED_CAPABILITY);
    }
    ids.sort();
    ids.into_iter()
        .map(|id| RequiredCapability {
            id: id.into(),
            version: 1,
        })
        .collect()
}
pub fn parse(contents: &str) -> RfResult<ProjectV4Read> {
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
        .filter(|c| {
            c.version != 1
                || ![
                    FLOW_PLAN_CAPABILITY,
                    SPLITTER_CAPABILITY,
                    PARTITIONED_CAPABILITY,
                ]
                .contains(&c.id.as_str())
        })
        .map(|c| format!("{} / {}", c.id, c.version))
        .collect();
    if !reasons.is_empty() {
        return readonly(contents, &wire.document, reasons);
    }
    let probe: SimulationProbe =
        serde_json::from_value(wire.simulation.clone()).map_err(json_error)?;
    if ![ACYCLIC_METHOD, PARTITIONED_METHOD].contains(&probe.steady_state_plan.method.id.as_str())
        || probe.steady_state_plan.method.version != 1
        || (probe.steady_state_plan.method.id == PARTITIONED_METHOD
            && probe
                .steady_state_plan
                .acceptance_contract_version
                .is_some_and(|v| v != 1))
    {
        reasons.push("unsupported steady-state method or acceptance contract".into());
    }
    if !reasons.is_empty() {
        return readonly(contents, &wire.document, reasons);
    }
    let engineering = match shared::decode(contents)? {
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
        ],
        "project",
    )?;
    let document = document::decode(&wire.document)?;
    let simulation: Simulation = serde_json::from_value(wire.simulation).map_err(json_error)?;
    let project = StoredProjectV4 {
        document,
        display_units: serde_json::from_value::<PresentationWire>(wire.presentation)
            .map_err(json_error)?
            .into_display_units()?,
        engineering,
        steady_state_plan: simulation.steady_state_plan,
    };
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
    let ProjectV4Read::Editable(p) = project else {
        return Err(RfError::invalid_input(
            "project v4 is read-only; saving is prohibited",
        ));
    };
    document::validate(&p.document)?;
    let wire = WriteWire {
        kind: STORED_PROJECT_FILE_KIND,
        schema_version: VERSION,
        required_capabilities: required_capabilities(p),
        document: &p.document,
        presentation: PresentationWire::from(&p.display_units),
        simulation: Simulation {
            steady_state_plan: p.steady_state_plan.clone(),
        },
        engineering: shared::encode(p.engineering.as_ref())?,
    };
    serde_json::to_string_pretty(&wire).map_err(json_error)
}
fn json_error(e: serde_json::Error) -> RfError {
    RfError::invalid_input(format!("project v4 JSON: {e}"))
}
#[cfg(test)]
mod tests;
