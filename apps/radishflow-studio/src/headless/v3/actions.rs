//! Strict execution DTOs. These are deliberately separate from storage records.
use super::{enums::UnitKind, protocol::*, records::*, strict::optional};
use crate::headless::protocol::{Field, Section, Value};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum RevisionRef {
    Number(u64),
    Step { step: String, output: Output },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedProject {
    pub document_id: String,
    pub file_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Confirmation {
    pub digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    pub unit: IdentityRef,
    pub port: String,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Inlet,
    Outlet,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowTarget {
    pub object: ObjectRef,
    pub section: Section,
    pub field: Field,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PackageSelection {
    Preferred {},
    Explicit { package_id: String },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowResources {
    pub cache_root: PathBuf,
    pub auth_cache_index: PathBuf,
    pub package: PackageSelection,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Adjustment {
    pub value_m: f64,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Adjustments {
    #[serde(
        default,
        deserialize_with = "optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub diameter: Option<Adjustment>,
    #[serde(
        default,
        deserialize_with = "optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub working_liquid_height: Option<Adjustment>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AdoptionTarget {
    New {
        tag: String,
        name: String,
        copy_case: bool,
    },
    Existing {
        task: IdentityRef,
        expected_revision: RevisionRef,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    RunFlow {
        expected_revision: RevisionRef,
    },
    Size {
        task: IdentityRef,
        expected_revision: RevisionRef,
    },
    Rate {
        task: IdentityRef,
        case: IdentityRef,
        expected_case_revision: RevisionRef,
        expected_geometry_revision: RevisionRef,
    },
    AdoptGeometry {
        source: IdentityRef,
        expected_revision: RevisionRef,
        target: AdoptionTarget,
        adjustments: Adjustments,
    },
    Write {
        expected_revision: RevisionRef,
        target: FlowTarget,
        value: Value,
    },
    CreateUnit {
        expected_revision: RevisionRef,
        unit_kind: UnitKind,
    },
    Connect {
        expected_revision: RevisionRef,
        source: Port,
        #[serde(
            default,
            deserialize_with = "optional",
            skip_serializing_if = "Option::is_none"
        )]
        sink: Option<Port>,
        #[serde(
            default,
            deserialize_with = "optional",
            skip_serializing_if = "Option::is_none"
        )]
        stream: Option<IdentityRef>,
    },
    Disconnect {
        expected_revision: RevisionRef,
        port: Port,
    },
    DisconnectStream {
        expected_revision: RevisionRef,
        stream: IdentityRef,
        #[serde(
            default,
            deserialize_with = "optional",
            skip_serializing_if = "Option::is_none"
        )]
        direction: Option<Direction>,
    },
    CreateSizingTask {},
    CreateRatingTask {},
    CreateSource {},
    CreateBasis {
        data: BasisData,
    },
    UpdateSizingTask {
        task: IdentityRef,
        expected_revision: RevisionRef,
        data: SizingData,
    },
    UpdateSource {
        source: IdentityRef,
        expected_revision: RevisionRef,
        data: SourceData,
    },
    UpdateBasis {
        basis: IdentityRef,
        expected_revision: RevisionRef,
        data: BasisData,
    },
    UpdateRatingLabels {
        task: IdentityRef,
        expected_revision: RevisionRef,
        data: Labels,
    },
    UpdateRatingGeometry {
        task: IdentityRef,
        expected_revision: RevisionRef,
        data: GeometryData,
    },
    UpdateRatingCase {
        task: IdentityRef,
        case: IdentityRef,
        expected_revision: RevisionRef,
        data: RatingCaseData,
    },
    AddRatingCase {
        task: IdentityRef,
        expected_revision: RevisionRef,
        #[serde(
            default,
            deserialize_with = "optional",
            skip_serializing_if = "Option::is_none"
        )]
        copy_from: Option<IdentityRef>,
    },
    ReorderRatingCases {
        task: IdentityRef,
        expected_revision: RevisionRef,
        cases: Vec<IdentityRef>,
    },
    DeleteEquipmentRecord {
        target: ObjectRef,
        expected_revision: RevisionRef,
    },
    CheckSizing {
        task: IdentityRef,
    },
    CheckRating {
        task: IdentityRef,
        case: IdentityRef,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub id: String,
    pub action: Action,
    #[serde(
        default,
        deserialize_with = "optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub confirmation: Option<Confirmation>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecuteRequest {
    pub schema_version: u32,
    pub project: PathBuf,
    pub expected_project: ExpectedProject,
    #[serde(
        default,
        deserialize_with = "optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub flow_resources: Option<FlowResources>,
    pub steps: Vec<Step>,
    pub queries: Vec<Query>,
}

impl Action {
    pub fn target(&self) -> Option<ObjectRef> {
        Some(match self {
            Self::Size { task, .. } => ObjectRef::Equipment(task.clone()),
            Self::Rate { task, case, .. } => ObjectRef::Case {
                task: task.clone(),
                case: case.clone(),
            },
            Self::AdoptGeometry { source, .. } => ObjectRef::Equipment(source.clone()),
            Self::Write { target, .. } => target.object.clone(),
            Self::Connect { source, .. } => ObjectRef::Unit(source.unit.clone()),
            Self::Disconnect { port, .. } => ObjectRef::Unit(port.unit.clone()),
            Self::DisconnectStream { stream, .. } => ObjectRef::Stream(stream.clone()),
            Self::UpdateSizingTask { task, .. }
            | Self::UpdateRatingLabels { task, .. }
            | Self::AddRatingCase { task, .. }
            | Self::ReorderRatingCases { task, .. }
            | Self::CheckSizing { task } => ObjectRef::Equipment(task.clone()),
            Self::UpdateRatingGeometry { task, .. } => ObjectRef::Geometry(task.clone()),
            Self::UpdateRatingCase { task, case, .. } | Self::CheckRating { task, case } => {
                ObjectRef::Case {
                    task: task.clone(),
                    case: case.clone(),
                }
            }
            Self::UpdateSource { source, .. } => ObjectRef::Source(source.clone()),
            Self::UpdateBasis { basis, .. } => ObjectRef::Basis(basis.clone()),
            Self::DeleteEquipmentRecord { target, .. } => target.clone(),
            _ => return None,
        })
    }
}
