//! Versioned review material. Only typed identity positions are normalized;
//! display strings and fixed historical evidence remain byte-for-byte meaningful.
use super::{
    actions::{FlowResources, Step},
    protocol::*,
    records,
    references::BoundOutput,
};
use rf_ui::{
    AppState,
    engineering::{EngineeringInputs, TaskImpact},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default, Clone)]
pub struct Aliases {
    identities: BTreeMap<(Output, String), Value>,
    generated_references: BTreeMap<String, (String, Value)>,
    adoptions: BTreeSet<String>,
}
impl Aliases {
    pub fn created(
        &mut self,
        step: &str,
        before: &AppState,
        after: &AppState,
        outputs: &BTreeMap<Output, BoundOutput>,
    ) {
        let old = before.workspace.engineering.inputs();
        let new = after.workspace.engineering.inputs();
        for (kind, value) in outputs {
            let created = match kind {
                Output::Task => {
                    !old.tasks.iter().any(|t| t.id().as_str() == value.id)
                        && new.tasks.iter().any(|t| t.id().as_str() == value.id)
                }
                Output::Case => !case_exists(old, &value.id) && case_exists(new, &value.id),
                Output::Source => {
                    !old.basis.sources.iter().any(|s| s.id.as_str() == value.id)
                        && new.basis.sources.iter().any(|s| s.id.as_str() == value.id)
                }
                Output::Basis => {
                    !old.basis
                        .basis_entries
                        .iter()
                        .any(|b| b.id.as_str() == value.id)
                        && new
                            .basis
                            .basis_entries
                            .iter()
                            .any(|b| b.id.as_str() == value.id)
                }
                Output::Unit => !before
                    .workspace
                    .document
                    .flowsheet
                    .units
                    .keys()
                    .any(|id| id.as_str() == value.id),
                Output::Stream => !before
                    .workspace
                    .document
                    .flowsheet
                    .streams
                    .keys()
                    .any(|id| id.as_str() == value.id),
                Output::Document | Output::Geometry => false,
            };
            if created {
                self.identities
                    .entry((*kind, value.id.clone()))
                    .or_insert_with(|| {
                        json!(IdentityRef::Step {
                            step: step.to_string(),
                            output: *kind,
                        })
                    });
            }
        }
    }
    pub fn identity(&self, kind: Output, id: &str) -> Value {
        self.identities
            .get(&(kind, id.to_string()))
            .cloned()
            .unwrap_or_else(|| json!(IdentityRef::id(id)))
    }
    fn identity_field(&self, v: &mut Value, field: &str, kind: Output) {
        if let Some(id) = v[field].as_str() {
            v[field] = self.identity(kind, id);
        }
    }
    fn reference(&self, v: &mut Value, kind: Output) {
        if let Some(id) = v["id"].as_str() {
            *v = self.identity(kind, id);
        }
    }
    fn binding(&self, v: &mut Value) {
        if v.get("source").is_some() {
            self.reference(&mut v["source"], Output::Source);
        }
        if v.get("basis").is_some() {
            self.reference(&mut v["basis"], Output::Basis);
        }
    }
    fn inputs(&self, v: &mut Value) {
        for field in ["inputs", "axial_sections"] {
            if let Some(map) = v[field].as_object_mut() {
                for binding in map.values_mut() {
                    self.binding(binding);
                }
            }
        }
        if let Some(list) = v["applicability_sources"].as_array_mut() {
            for r in &mut *list {
                self.reference(r, Output::Source);
            }
            sort_set(list);
        }
    }
    fn source(&self, mut v: Value) -> Value {
        if let Some((original, canonical)) = v["id"]
            .as_str()
            .and_then(|id| self.generated_references.get(id))
            && v["data"]["reference"].as_str() == Some(original.as_str())
        {
            v["data"]["reference"] = canonical.clone();
        }
        self.identity_field(&mut v, "id", Output::Source);
        if let Some(observations) = v["data"]["observations"].as_array_mut() {
            for o in &mut *observations {
                if let Some(dependencies) = o["depends_on"].as_array_mut() {
                    for r in &mut *dependencies {
                        self.reference(&mut r["source"], Output::Source);
                    }
                    sort_set(dependencies);
                }
            }
            sort_set(observations);
        }
        v
    }
    fn basis(&self, mut v: Value) -> Value {
        self.identity_field(&mut v, "id", Output::Basis);
        self.reference(&mut v["data"]["source"], Output::Source);
        v
    }
    fn task(&self, mut v: Value) -> Value {
        let live_adoption = v["id"]
            .as_str()
            .is_some_and(|id| self.adoptions.contains(id));
        self.identity_field(&mut v, "id", Output::Task);
        if v["kind"] == "sizing" {
            self.identity_field(&mut v, "case_id", Output::Case);
            self.inputs(&mut v["data"]);
            self.inputs(&mut v["data"]["case"]);
        } else {
            self.inputs(&mut v["geometry"]["data"]);
            // Only adoptions produced by this request share the current identity
            // namespace. Previously archived evidence remains untouched.
            if live_adoption && !v["geometry"]["adoption"].is_null() {
                self.adoption_evidence(&mut v["geometry"]["adoption"]);
            }
            if let Some(cases) = v["cases"].as_array_mut() {
                for c in cases {
                    self.identity_field(c, "id", Output::Case);
                    self.inputs(&mut c["data"]);
                }
            }
        }
        v
    }
    pub fn snapshot(&self, app: &AppState) -> Value {
        let i = app.workspace.engineering.inputs();
        let mut sources: Vec<_> = i
            .basis
            .sources
            .iter()
            .map(|v| self.source(records::source(v)))
            .collect();
        let mut basis: Vec<_> = i
            .basis
            .basis_entries
            .iter()
            .map(|v| self.basis(records::basis(v)))
            .collect();
        let mut tasks: Vec<_> = i
            .tasks
            .iter()
            .map(|v| self.task(records::task(v)))
            .collect();
        sort_set(&mut sources);
        sort_set(&mut basis);
        sort_set(&mut tasks);
        json!({"sources":sources,"basis":basis,"tasks":tasks})
    }
    pub fn impacts(&self, impacts: &[TaskImpact]) -> Value {
        let mut values: Vec<_> = impacts.iter().map(|i|json!({"task":self.identity(Output::Task,i.task_id.as_str()),"case":self.identity(Output::Case,i.case_id.as_str()),"input_changed":i.input_changed,"evidence_changed":i.evidence_changed})).collect();
        sort_set(&mut values);
        json!(values)
    }
}
fn case_exists(i: &EngineeringInputs, id: &str) -> bool {
    i.tasks.iter().any(|t| match t {
        rf_model::equipment::EquipmentTask::SeparatorSizing(t) => t.case.id.as_str() == id,
        rf_model::equipment::EquipmentTask::SeparatorRating(t) => {
            t.cases.iter().any(|c| c.id.as_str() == id)
        }
    })
}
fn sort_set(values: &mut [Value]) {
    values.sort_by_cached_key(Value::to_string);
}
fn normalize_numbers(v: &mut Value) {
    match v {
        Value::Number(n) if n.is_f64() && n.as_f64() == Some(0.0) => *v = json!(0.0),
        Value::Array(values) => values.iter_mut().for_each(normalize_numbers),
        Value::Object(values) => values.values_mut().for_each(normalize_numbers),
        _ => {}
    }
}
pub fn digest(
    file_sha256: &str,
    prefix: &[Step],
    resources: Option<&FlowResources>,
    preview: &Value,
) -> Result<String, Failure> {
    let actions: Vec<_> = prefix
        .iter()
        .map(|s| {
            let mut action = json!(s.action);
            normalize_action_sets(&mut action);
            json!({"id":s.id,"action":action})
        })
        .collect();
    let mut review = json!({"namespace":"af4c-confirm-v1","file_sha256":file_sha256,"steps":actions,"flow_resources":resources,"preview":preview});
    normalize_numbers(&mut review);
    let mut bytes = Vec::new();
    canonical(&review, &mut bytes).map_err(super::invalid_request)?;
    Ok(format!("af4c-confirm-v1:{}", rf_store::sha256_hex(&bytes)))
}

// The protocol contains no arbitrary object-valued user data. These fields are
// semantic sets in the input DTO; steps and Rating members remain ordered.
fn normalize_action_sets(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                normalize_action_sets(value);
                if matches!(
                    key.as_str(),
                    "applicability_sources" | "depends_on" | "observations"
                ) && let Value::Array(values) = value
                {
                    sort_set(values);
                }
            }
        }
        Value::Array(values) => values.iter_mut().for_each(normalize_action_sets),
        _ => {}
    }
}
// Sort explicitly, independent of serde_json feature unification in consumers.
fn canonical(value: &Value, output: &mut Vec<u8>) -> Result<(), serde_json::Error> {
    match value {
        Value::Object(map) => {
            output.push(b'{');
            let sorted: BTreeMap<_, _> = map.iter().collect();
            for (index, (key, value)) in sorted.into_iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                serde_json::to_writer(&mut *output, key)?;
                output.push(b':');
                canonical(value, output)?;
            }
            output.push(b'}');
        }
        Value::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                canonical(value, output)?;
            }
            output.push(b']');
        }
        value => serde_json::to_writer(&mut *output, value)?,
    }
    Ok(())
}

impl Aliases {
    /// Use the domain allocation map, never generated-string patterns. Copies
    /// are named by the source they captured; the design source is a single output.
    pub fn adoption(
        &mut self,
        step: &str,
        proposal: &rf_ui::engineering::PreparedGeometryAdoption,
        app: &AppState,
    ) {
        self.adoptions
            .insert(proposal.target().task_id().to_string());
        for (original, copy) in proposal.source_copies() {
            let identity = json!({"step":step,"output":"copied_source","source":self.identity(Output::Source,original.as_str())});
            self.identities
                .insert((Output::Source, copy.to_string()), identity);
            if let Some(reference) = self.generated_references.get(original.as_str()).cloned() {
                self.generated_references
                    .insert(copy.to_string(), reference);
            }
        }
        let id = proposal.design_source();
        self.identities.insert(
            (Output::Source, id.to_string()),
            json!({"step":step,"output":"source"}),
        );
        let record = proposal
            .inputs()
            .basis
            .sources
            .iter()
            .find(|s| s.id == *id)
            .expect("prepared design source");
        self.generated_references.insert(id.to_string(),(record.reference.clone(),json!({
            "generated_from":{"document":app.workspace.document.metadata.document_id.as_str(),"task":self.identity(Output::Task,proposal.source_task().as_str())}
        })));
    }
    fn adoption_evidence(&self, value: &mut Value) {
        value["source_task"] = self.task(value["source_task"].take());
        self.source_array(&mut value["source_basis"]["sources"]);
        self.basis_array(&mut value["source_basis"]["basis"]);
    }
    fn source_array(&self, value: &mut Value) {
        if let Some(values) = value.as_array_mut() {
            for v in &mut *values {
                *v = self.source(v.take());
            }
            sort_set(values);
        }
    }
    fn basis_array(&self, value: &mut Value) {
        if let Some(values) = value.as_array_mut() {
            for v in &mut *values {
                *v = self.basis(v.take());
            }
            sort_set(values);
        }
    }
    pub fn candidate(
        &self,
        candidate: &rf_ui::engineering::editor::separator_sizing::SeparatorCandidate,
    ) -> Value {
        let mut value = super::results::sizing(candidate);
        value["capture"]["task"] = self.task(value["capture"]["task"].take());
        self.source_array(&mut value["capture"]["sources"]);
        self.basis_array(&mut value["capture"]["basis"]);
        for field in ["assumed_sources", "pending_review_sources"] {
            if let Some(values) = value[field].as_array_mut() {
                for v in &mut *values {
                    *v = self.identity(Output::Source, v.as_str().expect("source id"));
                }
                sort_set(values);
            }
        }
        if let Some(issues) = value["height_issues"].as_array_mut() {
            for issue in &mut *issues {
                if issue["target"]["kind"] == "source" {
                    self.identity_field(&mut issue["target"], "id", Output::Source);
                }
                if issue["reason"]["code"] == "missing_source" {
                    self.identity_field(&mut issue["reason"], "id", Output::Source);
                }
            }
            sort_set(issues);
        }
        value
    }
}

pub fn digest_planned(
    version: u32,
    file_sha256: &str,
    review: &Value,
    resources: Option<&FlowResources>,
    preview: &Value,
) -> Result<String, Failure> {
    let namespace = format!("headless-v{version}-confirm-v1");
    let mut material = json!({"namespace":namespace,"file_sha256":file_sha256,"review":review,"flow_resources":resources,"preview":preview});
    normalize_numbers(&mut material);
    let mut bytes = Vec::new();
    canonical(&material, &mut bytes).map_err(super::invalid_request)?;
    Ok(format!("{namespace}:{}", rf_store::sha256_hex(&bytes)))
}
