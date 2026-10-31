//! Owned, read-only projections for consumers that outlive one workspace borrow.
use rf_ui::{DiagnosticSummary, FlowResultRecord, WorkspaceState, result_query::*};
use std::sync::Arc;

/// Session-local evidence. Never serialize this guard or substitute document IDs
/// for its opaque qualification; even a reopened identical file is a new owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowResultGuard(Arc<FlowResultIdentity>);

#[derive(Debug, PartialEq, Eq)]
enum FlowResultIdentity {
    Unavailable,
    Current {
        origin: FlowResultOrigin,
        snapshot_id: String,
        sequence: u64,
        payload: FlowResultPayload,
    },
    Failed {
        origin: FlowResultOrigin,
        diagnostic: DiagnosticSummary,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlowResultPayload {
    Acyclic,
    Partitioned,
    Simultaneous,
}
impl FlowResultPayload {
    fn of(snapshot: &FlowResultRecord) -> Self {
        match snapshot {
            FlowResultRecord::Acyclic(_) => Self::Acyclic,
            FlowResultRecord::Partitioned(_) => Self::Partitioned,
            FlowResultRecord::Simultaneous(_) => Self::Simultaneous,
        }
    }
}
impl FlowResultGuard {
    pub fn from_snapshot(snapshot: &FlowResultRecord) -> Option<Self> {
        snapshot.origin.as_ref().map(|origin| {
            Self(Arc::new(FlowResultIdentity::Current {
                origin: origin.clone(),
                snapshot_id: snapshot.id.as_str().into(),
                sequence: snapshot.sequence,
                payload: FlowResultPayload::of(snapshot),
            }))
        })
    }
    pub(crate) fn unavailable() -> Self {
        Self(Arc::new(FlowResultIdentity::Unavailable))
    }

    pub fn is_current(&self, workspace: &WorkspaceState) -> bool {
        match (self.0.as_ref(), workspace.flow_result()) {
            (
                FlowResultIdentity::Current {
                    origin: expected,
                    snapshot_id,
                    sequence,
                    payload,
                },
                FlowResultView::Current { snapshot, origin },
            ) => {
                expected == origin
                    && snapshot_id == snapshot.id.as_str()
                    && *sequence == snapshot.sequence
                    && *payload == FlowResultPayload::of(snapshot)
            }
            (
                FlowResultIdentity::Failed {
                    origin: expected,
                    diagnostic: expected_diagnostic,
                },
                FlowResultView::Failed { diagnostic, origin },
            ) => expected == origin && expected_diagnostic == diagnostic,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowResultReview {
    pub status: ResultStatus,
    pub origin: Option<FlowResultOrigin>,
    pub unavailable: Option<FlowResultUnavailable>,
    pub guard: Option<FlowResultGuard>,
}
impl FlowResultReview {
    pub fn capture(workspace: &WorkspaceState) -> Self {
        let view = workspace.flow_result();
        let status = view.status();
        let (origin, unavailable, guard) = match view {
            FlowResultView::Current { snapshot, origin } => (
                Some(origin.clone()),
                None,
                FlowResultGuard::from_snapshot(snapshot),
            ),
            FlowResultView::Failed { diagnostic, origin } => (
                Some(origin.clone()),
                None,
                Some(FlowResultGuard(Arc::new(FlowResultIdentity::Failed {
                    origin: origin.clone(),
                    diagnostic: diagnostic.clone(),
                }))),
            ),
            FlowResultView::Stale { origin, reason } => (origin.cloned(), Some(reason), None),
            FlowResultView::Missing { reason } => (None, Some(reason), None),
        };
        Self {
            status,
            origin,
            unavailable,
            guard,
        }
    }
}

/// Technical evidence is supplementary to the localized summary. Opaque attempt
/// identities remain in memory; output records their availability, not pointers.
pub fn flow_origin_lines(origin: Option<&FlowResultOrigin>) -> Vec<(String, String)> {
    let Some(origin) = origin else {
        return vec![("execution_origin".into(), "unrecorded".into())];
    };
    let mut rows = vec![
        ("document_id".into(), origin.document_id().as_str().into()),
        ("input_revision".into(), origin.revision().to_string()),
    ];
    let Some(execution) = origin.execution() else {
        rows.push(("execution_origin".into(), "unrecorded".into()));
        return rows;
    };
    rows.extend([
        (
            "execution_identity".into(),
            "recorded; session-local, not serialized".into(),
        ),
        (
            "solver_implementation".into(),
            execution.solver_implementation.into(),
        ),
    ]);
    if let Some(plan) = &execution.plan {
        rows.extend([
            ("solver_method".into(), plan.method.into()),
            (
                "acceptance_contract_version".into(),
                plan.acceptance_contract_version.to_string(),
            ),
            (
                "plan_configuration_sha256".into(),
                plan.configuration_sha256.clone(),
            ),
        ]);
        if let Some(modular) = &plan.modular {
            rows.extend([
                ("solver_capability".into(), modular.capability.into()),
                (
                    "solver_capability_version".into(),
                    modular.capability_version.to_string(),
                ),
                (
                    "response_contract_version".into(),
                    modular.response_contract_version.to_string(),
                ),
                (
                    "core_method".into(),
                    modular.core_method.unwrap_or("not invoked").into(),
                ),
            ]);
        }
    }
    let Some(package) = &execution.package else {
        rows.push(("package".into(), "not loaded during preparation".into()));
        return rows;
    };
    rows.extend([
        ("package_id".into(), package.package_id.clone()),
        ("package_version".into(), package.version.clone()),
        (
            "package_origin".into(),
            match package.origin {
                ResultDataOrigin::InMemory => "in_memory",
                ResultDataOrigin::Cache => "cache",
                ResultDataOrigin::LocalFiles => "local_files",
                ResultDataOrigin::Bundled => "bundled",
            }
            .into(),
        ),
        ("liquid_model".into(), package.liquid_model.into()),
        ("vapor_model".into(), package.vapor_model.into()),
        (
            "payload_fingerprint".into(),
            package.payload_fingerprint.clone(),
        ),
        (
            "payload_size_bytes".into(),
            package.canonical_payload_size_bytes.to_string(),
        ),
        (
            "manifest_schema_version".into(),
            package.manifest_schema_version.to_string(),
        ),
        (
            "payload_schema_version".into(),
            package.payload_schema_version.to_string(),
        ),
        (
            "thermo_implementation".into(),
            package.thermo_implementation.into(),
        ),
        (
            "flash_implementation".into(),
            package.flash_implementation.into(),
        ),
    ]);
    rows
}

pub fn equipment_origin_lines(
    origin: &rf_ui::engineering::EquipmentResultOrigin,
) -> Vec<(String, String)> {
    let mut rows = vec![
        ("documentId".into(), origin.document_id.as_str().into()),
        (
            "executionIdentity".into(),
            "recorded; session-local, not serialized".into(),
        ),
        ("taskId".into(), origin.member.task_id.as_str().into()),
        ("caseId".into(), origin.member.case_id.as_str().into()),
        ("methodId".into(), origin.method.id.clone()),
        ("methodVersion".into(), origin.method.version.to_string()),
        ("taskRevision".into(), origin.task_revision.to_string()),
        (
            "caseRevision".into(),
            origin
                .case_revision
                .map(|v| v.to_string())
                .unwrap_or_else(|| "notApplicable".into()),
        ),
        (
            "geometryRevision".into(),
            origin
                .geometry_revision
                .map(|v| v.to_string())
                .unwrap_or_else(|| "notApplicable".into()),
        ),
        (
            "inputGeneration".into(),
            origin.generation.input.to_string(),
        ),
        (
            "evidenceGeneration".into(),
            origin.generation.evidence.to_string(),
        ),
        ("taskTag".into(), origin.task_tag.clone()),
        ("taskName".into(), origin.task_name.clone()),
        ("caseName".into(), origin.case_name.clone()),
    ];
    rows.extend(
        origin
            .basis_revisions
            .iter()
            .map(|(id, rev)| (format!("basisRevision.{}", id.as_str()), rev.to_string())),
    );
    rows.extend(
        origin
            .source_revisions
            .iter()
            .map(|(id, rev)| (format!("sourceRevision.{}", id.as_str()), rev.to_string())),
    );
    rows
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentResultGuard {
    origin: rf_ui::engineering::EquipmentResultOrigin,
    status: rf_ui::engineering::EquipmentResultStatus,
}
impl EquipmentResultGuard {
    pub fn capture(
        session: &rf_ui::engineering::EngineeringSession,
        member: &rf_ui::engineering::EquipmentCaseKey,
    ) -> Option<Self> {
        use rf_ui::engineering::EquipmentResultStatus;
        let view = session.query_case_result(member);
        matches!(
            view.status(),
            EquipmentResultStatus::Current | EquipmentResultStatus::Failed
        )
        .then(|| view.origin().cloned())
        .flatten()
        .map(|origin| Self {
            origin,
            status: view.status(),
        })
    }
    pub fn is_current(&self, session: &rf_ui::engineering::EngineeringSession) -> bool {
        let view = session.query_case_result(&self.origin.member);
        view.status() == self.status && view.origin() == Some(&self.origin)
    }
}
