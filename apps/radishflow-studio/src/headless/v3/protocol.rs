use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum IdentityRef {
    Id { id: String },
    Step { step: String, output: Output },
}
impl IdentityRef {
    pub fn id(id: impl ToString) -> Self {
        Self::Id { id: id.to_string() }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Output {
    Document,
    Unit,
    Stream,
    Task,
    Case,
    Geometry,
    Basis,
    Source,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ObjectRef {
    Unit(IdentityRef),
    Stream(IdentityRef),
    Equipment(IdentityRef),
    Case {
        task: IdentityRef,
        case: IdentityRef,
    },
    Geometry(IdentityRef),
    Basis(IdentityRef),
    Source(IdentityRef),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Scope {
    Project,
    Flow,
    Object(ObjectRef),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum QueryKind {
    Objects {},
    Fields { target: ObjectRef },
    Record { target: ObjectRef },
    Actions { scope: Scope },
    Results { scope: Scope },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub id: String,
    pub query: QueryKind,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryRequest {
    pub schema_version: u32,
    pub project: PathBuf,
    pub queries: Vec<Query>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ok,
    Error,
    Blocked,
    Failed,
    ConfirmationRequired,
}
impl Status {
    pub fn exit_code(self) -> i32 {
        match self {
            Self::Ok => 0,
            Self::Error => 2,
            Self::Blocked => 3,
            Self::Failed => 4,
            Self::ConfirmationRequired => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Request,
    Project,
    Step,
    Query,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Code {
    UnsupportedVersion,
    InvalidRequest,
    ProjectIo,
    InvalidProject,
    DifferentDocument,
    UnsupportedCapability,
    UnknownReference,
    ReferenceKindMismatch,
    ObjectMissing,
    FieldMissing,
    InputContextChanged,
    ProjectConflict,
    RecordConflict,
    ReadOnlyProject,
    ConfirmationRequired,
    ConfirmationMismatch,
    ActionRejected,
    RunFailed,
    RunStartFailed,
}

#[derive(Debug, Clone, Serialize)]
pub struct Failure {
    pub stage: Stage,
    pub code: Code,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<Box<ObjectRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Box<Value>>,
    #[serde(skip)]
    pub status: Status,
}
impl Failure {
    pub fn new(stage: Stage, code: Code, message: impl Into<String>) -> Self {
        Self {
            stage,
            code,
            message: message.into(),
            step_id: None,
            query_id: None,
            target: None,
            details: None,
            status: Status::Error,
        }
    }
    pub fn target(mut self, target: &ObjectRef) -> Self {
        self.target = Some(Box::new(target.clone()));
        self
    }
}

#[derive(Debug, Serialize)]
pub struct ProjectSnapshot {
    pub document_id: String,
    pub project_schema_version: u32,
    pub flow_revision: u64,
    pub file_sha256: String,
    pub read_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only_reason: Option<Code>,
    pub equipment_task_count: usize,
}

#[derive(Debug, Serialize)]
pub struct QueryReply {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Failure>,
}
#[derive(Debug, Serialize)]
pub struct Response {
    pub schema_version: u32,
    pub status: Status,
    pub project: Option<ProjectSnapshot>,
    pub steps: Vec<Value>,
    pub queries: Vec<QueryReply>,
    pub error: Option<Failure>,
}
impl Default for Response {
    fn default() -> Self {
        Self {
            schema_version: 3,
            status: Status::Ok,
            project: None,
            steps: Vec::new(),
            queries: Vec::new(),
            error: None,
        }
    }
}
impl Response {
    pub fn fail(&mut self, failure: Failure) {
        if self.error.is_none() {
            self.status = failure.status;
            self.error = Some(failure);
        }
    }
}
