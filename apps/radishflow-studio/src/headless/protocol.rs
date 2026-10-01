//! Local CLI wire types. They do not add serialization to application identities or commands.
use rf_ui::variable_browser as browser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRequest {
    pub schema_version: u32,
    pub project: PathBuf,
    pub document_id: String,
    pub expected_revision: u64,
    pub cache_root: PathBuf,
    pub auth_cache_index: PathBuf,
    /// Null or omitted uses the document's normal Preferred selection.
    pub package_id: Option<String>,
    pub writes: Vec<Write>,
    pub reads: Vec<Target>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Write {
    pub target: Target,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub object: Object,
    pub section: Section,
    pub field: Field,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Object {
    Unit(String),
    Stream(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section {
    Inputs,
    Results,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Field {
    Name,
    OutletTemperature,
    OutletPressure,
    Temperature,
    Pressure,
    MolarFlow,
    MoleFraction(String),
    PhaseFraction(String),
    PhaseMoleFraction { phase: String, component: String },
    PhaseMolarEnthalpy(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Number(f64),
    Text(String),
}

impl Target {
    pub fn variable_id(&self, document: &rf_ui::DocumentId) -> browser::VariableId {
        browser::VariableId {
            document: document.clone(),
            object: match &self.object {
                Object::Unit(id) => browser::ObjectId::Unit(id.clone().into()),
                Object::Stream(id) => browser::ObjectId::Stream(id.clone().into()),
            },
            section: match self.section {
                Section::Inputs => browser::VariableSection::Inputs,
                Section::Results => browser::VariableSection::Results,
            },
            field: match &self.field {
                Field::Name => browser::VariableField::Name,
                Field::OutletTemperature => browser::VariableField::OutletTemperature,
                Field::OutletPressure => browser::VariableField::OutletPressure,
                Field::Temperature => browser::VariableField::Temperature,
                Field::Pressure => browser::VariableField::Pressure,
                Field::MolarFlow => browser::VariableField::MolarFlow,
                Field::MoleFraction(id) => browser::VariableField::MoleFraction(id.clone().into()),
                Field::PhaseFraction(phase) => browser::VariableField::PhaseFraction(phase.clone()),
                Field::PhaseMoleFraction { phase, component } => {
                    browser::VariableField::PhaseMoleFraction {
                        phase: phase.clone(),
                        component: component.clone().into(),
                    }
                }
                Field::PhaseMolarEnthalpy(phase) => {
                    browser::VariableField::PhaseMolarEnthalpy(phase.clone())
                }
            },
        }
    }
}

impl From<browser::VariableId> for Target {
    fn from(id: browser::VariableId) -> Self {
        Self {
            object: match id.object {
                browser::ObjectId::Unit(id) => Object::Unit(id.into_inner()),
                browser::ObjectId::Stream(id) => Object::Stream(id.into_inner()),
            },
            section: match id.section {
                browser::VariableSection::Inputs => Section::Inputs,
                browser::VariableSection::Results => Section::Results,
            },
            field: match id.field {
                browser::VariableField::Name => Field::Name,
                browser::VariableField::OutletTemperature => Field::OutletTemperature,
                browser::VariableField::OutletPressure => Field::OutletPressure,
                browser::VariableField::Temperature => Field::Temperature,
                browser::VariableField::Pressure => Field::Pressure,
                browser::VariableField::MolarFlow => Field::MolarFlow,
                browser::VariableField::MoleFraction(id) => Field::MoleFraction(id.into_inner()),
                browser::VariableField::PhaseFraction(phase) => Field::PhaseFraction(phase),
                browser::VariableField::PhaseMoleFraction { phase, component } => {
                    Field::PhaseMoleFraction {
                        phase,
                        component: component.into_inner(),
                    }
                }
                browser::VariableField::PhaseMolarEnthalpy(phase) => {
                    Field::PhaseMolarEnthalpy(phase)
                }
            },
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ok,
    Error,
    Blocked,
    Failed,
}
impl Status {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Ok => 0,
            Self::Error => 2,
            Self::Blocked => 3,
            Self::Failed => 4,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub schema_version: u32,
    pub status: Status,
    pub document: Option<Document>,
    pub package_id: Option<String>,
    pub writes: Vec<WriteReceipt>,
    pub variables: Vec<Variable>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<Vec<super::workflow::StepReceipt>>,
    pub diagnostic: Option<Diagnostic>,
    pub error: Option<Failure>,
}
impl Default for Response {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            status: Status::Ok,
            document: None,
            package_id: None,
            writes: Vec::new(),
            variables: Vec::new(),
            steps: None,
            diagnostic: None,
            error: None,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Document {
    pub id: String,
    pub revision: u64,
}
#[derive(Debug, Serialize)]
pub struct WriteReceipt {
    pub target: Target,
    pub revision: u64,
    pub changed: bool,
}
#[derive(Debug, Serialize)]
pub struct Failure {
    pub stage: &'static str,
    pub code: &'static str,
    pub message: String,
    pub index: Option<usize>,
}
#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub code: Option<String>,
    pub message: String,
    pub unit_ids: Vec<String>,
    pub stream_ids: Vec<String>,
    pub ports: Vec<Port>,
}
#[derive(Debug, Serialize)]
pub struct Port {
    pub unit_id: String,
    pub port: String,
}
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Source {
    DocumentInput,
    StreamTemplate,
    SolveResult {
        snapshot: String,
        revision: u64,
        sequence: u64,
    },
    NoResult,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueState {
    Valid,
    Invalid,
    Unspecified,
    Missing,
    Stale,
}
#[derive(Debug, Serialize)]
pub struct Variable {
    pub target: Target,
    pub label: String,
    pub unit: &'static str,
    pub value: Option<Value>,
    pub state: ValueState,
    pub source: Source,
    pub writable: bool,
}
impl From<browser::VariableDescriptor> for Variable {
    fn from(row: browser::VariableDescriptor) -> Self {
        let unit = row.unit_symbol();
        Self {
            target: row.id.into(),
            label: row.label,
            unit,
            value: row.value.map(|v| match v {
                browser::VariableValue::Number(n) => Value::Number(n),
                browser::VariableValue::Text(t) => Value::Text(t),
            }),
            state: match row.state {
                browser::ValueState::Valid => ValueState::Valid,
                browser::ValueState::Invalid => ValueState::Invalid,
                browser::ValueState::Unspecified => ValueState::Unspecified,
                browser::ValueState::Missing => ValueState::Missing,
                browser::ValueState::Stale => ValueState::Stale,
            },
            source: match row.source {
                browser::ValueSource::DocumentInput => Source::DocumentInput,
                browser::ValueSource::StreamTemplate => Source::StreamTemplate,
                browser::ValueSource::NoResult => Source::NoResult,
                browser::ValueSource::SolveResult {
                    snapshot,
                    revision,
                    sequence,
                } => Source::SolveResult {
                    snapshot: snapshot.as_str().to_string(),
                    revision,
                    sequence,
                },
            },
            writable: row.write_via.is_some(),
        }
    }
}
