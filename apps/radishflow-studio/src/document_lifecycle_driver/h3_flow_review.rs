//! Session-local H3 review of two streams from one accepted flow solve.

pub(crate) mod bundled_water;
mod qualification;
mod save;
#[cfg(test)]
mod tests;

use super::{StudioH3Review, flow_plan::app_state_from_v4};
use crate::{
    FlowResultGuard, build_workspace_solve_request, solve_workspace_with_property_package,
};
pub use qualification::{H3PureWaterQualification, H3WaterPropertySource};
pub use rf_model::equipment::ConcentricTubeFlowStream as H3FlowStream;
use rf_model::equipment::{ConcentricTubeLiquidInlet, ConcentricTubeRatingTask};
use rf_property_package::PropertyPackageProvider;
use rf_store::project_v4::{
    self, EquipmentSupport, FlowPlanSupport, ProjectV4Read, StoredFlowPlan, StoredProjectV4,
};
use rf_types::{RfError, RfResult, StreamId};
use rf_ui::{
    AppState, FlowExecutionQualification, FlowResultRecord, SolveSnapshotId, WorkspaceState,
    engineering::{
        EquipmentCaseKey,
        concentric_tube::{ConcentricTubeRating, RatingRevisions, SavedRatingError},
    },
    result_query::{FlowResultOrigin, FlowResultView},
    variable_browser::ObjectId,
};
pub use save::PreparedH3FlowAssociationSave;
use std::path::Path;

pub(crate) const FLOW_SUPPORT: FlowPlanSupport = FlowPlanSupport::Sequential;
pub(crate) const EQUIPMENT_SUPPORT: EquipmentSupport =
    EquipmentSupport::ConcentricTubeFlowAssociation;

#[derive(Debug)]
pub struct H3FlowPreview {
    guard: FlowResultGuard,
    origin: FlowResultOrigin,
    snapshot_id: SolveSnapshotId,
    sequence: u64,
    project_sha256: String,
    member: EquipmentCaseKey,
    revisions: RatingRevisions,
    hot: H3FlowStream,
    cold: H3FlowStream,
    hot_inlet: ConcentricTubeLiquidInlet,
    cold_inlet: ConcentricTubeLiquidInlet,
}

impl H3FlowPreview {
    pub fn origin(&self) -> &FlowResultOrigin {
        &self.origin
    }

    pub fn snapshot_id(&self) -> &SolveSnapshotId {
        &self.snapshot_id
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn project_sha256(&self) -> &str {
        &self.project_sha256
    }

    pub fn member(&self) -> &EquipmentCaseKey {
        &self.member
    }

    pub fn hot(&self) -> (&H3FlowStream, ConcentricTubeLiquidInlet) {
        (&self.hot, self.hot_inlet)
    }

    pub fn cold(&self) -> (&H3FlowStream, ConcentricTubeLiquidInlet) {
        (&self.cold, self.cold_inlet)
    }
}

/// Consumers must recheck a result against its review session before reading it.
#[derive(Debug)]
pub struct H3FlowRating {
    preview: H3FlowPreview,
    rating: ConcentricTubeRating,
}

/// Owns the original H3 inputs and a private flow runtime assembled from the
/// same v4 read. Only an explicit association save can write project data.
#[derive(Debug)]
pub struct H3FlowReview {
    saved: StudioH3Review,
    flow: AppState,
    saved_flow_input: FlowExecutionQualification,
    qualification: H3PureWaterQualification,
}

impl H3FlowReview {
    pub fn open(path: &Path, qualification: H3PureWaterQualification) -> RfResult<Self> {
        qualification.validate()?;
        let path = std::path::absolute(path)
            .map_err(|error| RfError::invalid_input(format!("resolve project path: {error}")))?;
        let contents = std::fs::read_to_string(&path).map_err(|error| {
            RfError::invalid_input(format!("read project {}: {error}", path.display()))
        })?;
        if rf_store::project_v3::probe_version(&contents)? != project_v4::VERSION {
            return Err(rejected(
                "unsupported_project",
                "H3 flow review requires a v4 project",
            ));
        }
        let project = parse_project(&contents)?;
        Self::assemble(&path, *project, &contents, qualification)
    }

    /// Assemble a previously parsed project with the same bytes used for its fingerprint.
    pub(crate) fn from_saved_project(
        path: &Path,
        project: StoredProjectV4,
        contents: &str,
        qualification: H3PureWaterQualification,
    ) -> RfResult<Self> {
        qualification.validate()?;
        let path = std::path::absolute(path)
            .map_err(|error| RfError::invalid_input(format!("resolve project path: {error}")))?;
        Self::assemble(&path, project, contents, qualification)
    }

    fn assemble(
        path: &Path,
        mut project: StoredProjectV4,
        contents: &str,
        qualification: H3PureWaterQualification,
    ) -> RfResult<Self> {
        if !matches!(project.steady_state_plan, StoredFlowPlan::Acyclic)
            || project.object_identities.is_none()
        {
            return Err(rejected(
                "unsupported_project",
                "H3 flow review requires an acyclic plan and saved object identities",
            ));
        }
        let saved = StudioH3Review::take(
            &project.document,
            &project.display_units,
            project_v4::VERSION,
            &mut project.engineering,
            contents,
        )
        .ok_or_else(|| rejected("case_missing", "no saved concentric-tube rating case"))?;
        let flow = app_state_from_v4(project, path)?;
        let saved_flow_input = flow.workspace.flow_execution_qualification();
        Ok(Self {
            saved,
            flow,
            saved_flow_input,
            qualification,
        })
    }

    pub fn workspace(&self) -> &WorkspaceState {
        &self.flow.workspace
    }

    pub fn cases(&self) -> impl Iterator<Item = &ConcentricTubeRatingTask> {
        self.saved.cases()
    }

    pub fn solve_flow(&mut self, provider: &impl PropertyPackageProvider) -> RfResult<()> {
        let package = self
            .workspace()
            .document
            .flowsheet
            .property_package_id()
            .ok_or_else(|| rejected("package_unqualified", "no flowsheet property package"))?;
        let request = build_workspace_solve_request(&self.flow, package)?;
        solve_workspace_with_property_package(&mut self.flow, provider, &request)
    }

    pub fn preview(
        &self,
        member: EquipmentCaseKey,
        hot: H3FlowStream,
        cold: H3FlowStream,
    ) -> RfResult<H3FlowPreview> {
        let task = self
            .saved
            .case(&member)
            .ok_or_else(|| rejected("case_missing", "saved H3 task / case does not exist"))?;
        if task
            .case
            .flow_association
            .as_ref()
            .is_some_and(|saved| saved.hot != hot || saved.cold != cold)
        {
            return Err(rejected(
                "association_mismatch",
                "the selected streams differ from the saved association",
            ));
        }
        if hot.id == cold.id {
            return Err(
                rejected("same_stream", "hot and cold require distinct streams")
                    .with_related_stream_id(hot.id),
            );
        }
        let flow = self.workspace().flow_result();
        let FlowResultView::Current { snapshot, origin } = flow else {
            return Err(rejected(
                "flow_unavailable",
                format!(
                    "a Current accepted flow result is required (found {:?})",
                    flow.status()
                ),
            ));
        };
        if !matches!(snapshot, FlowResultRecord::Acyclic(_)) {
            return Err(rejected(
                "flow_unavailable",
                "an acyclic flow result is required",
            ));
        }
        self.qualification.check_package(origin)?;
        let guard = FlowResultGuard::from_snapshot(snapshot)
            .ok_or_else(|| rejected("flow_unavailable", "flow result has no origin"))?;
        let hot_inlet = self.inlet(snapshot, &hot, &task.case.hot.properties, Side::Hot)?;
        let cold_inlet = self.inlet(snapshot, &cold, &task.case.cold.properties, Side::Cold)?;
        Ok(H3FlowPreview {
            guard,
            origin: origin.clone(),
            snapshot_id: snapshot.id.clone(),
            sequence: snapshot.sequence,
            project_sha256: self.saved.file_sha256().into(),
            member,
            revisions: task.into(),
            hot,
            cold,
            hot_inlet,
            cold_inlet,
        })
    }

    /// Resolve the saved pair only against a newly accepted Current result.
    pub fn preview_saved(&self, member: EquipmentCaseKey) -> RfResult<H3FlowPreview> {
        let saved = self
            .saved
            .case(&member)
            .ok_or_else(|| rejected("case_missing", "saved H3 task / case does not exist"))?
            .case
            .flow_association
            .as_ref()
            .ok_or_else(|| {
                rejected(
                    "association_missing",
                    "the H3 case has no saved flow association",
                )
            })?;
        self.preview(member, saved.hot.clone(), saved.cold.clone())
    }

    /// Run only for the exact reviewed pair and input set. The effective
    /// inlets live in a temporary task and never replace the saved inputs.
    pub fn rate(&self, preview: H3FlowPreview) -> RfResult<H3FlowRating> {
        let mut task = self.current_case(&preview)?.clone();
        task.case.flow_association = None;
        task.case.hot.inlet = preview.hot_inlet;
        task.case.cold.inlet = preview.cold_inlet;
        let rating = rf_ui::engineering::concentric_tube::rate_saved_case(
            &task,
            self.saved.basis(),
            preview.revisions,
        )
        .map_err(|error| match error {
            SavedRatingError::Calculation(error) => error,
            SavedRatingError::RevisionConflict { .. } => inputs_changed(),
        })?;
        self.qualification.check_outlet(
            Side::Hot,
            &preview.hot.id,
            rating.hot.outlet.temperature_k,
            rating.hot.outlet.pressure_pa,
        )?;
        self.qualification.check_outlet(
            Side::Cold,
            &preview.cold.id,
            rating.cold.outlet.temperature_k,
            rating.cold.outlet.pressure_pa,
        )?;
        self.current_case(&preview)?;
        Ok(H3FlowRating { preview, rating })
    }

    pub fn current_rating<'a>(
        &'a self,
        result: &'a H3FlowRating,
    ) -> RfResult<&'a ConcentricTubeRating> {
        self.current_case(&result.preview)?;
        Ok(&result.rating)
    }

    fn current_case(&self, preview: &H3FlowPreview) -> RfResult<&ConcentricTubeRatingTask> {
        if preview.project_sha256 != self.saved.file_sha256()
            || !preview.guard.is_current(self.workspace())
        {
            return Err(inputs_changed());
        }
        self.check_stream(&preview.hot, Side::Hot)?;
        self.check_stream(&preview.cold, Side::Cold)?;
        self.saved
            .case(&preview.member)
            .filter(|task| RatingRevisions::from(*task) == preview.revisions)
            .ok_or_else(inputs_changed)
    }

    fn check_stream(&self, stream: &H3FlowStream, side: Side) -> RfResult<()> {
        let object = ObjectId::Stream(stream.id.clone());
        if self.workspace().object_creation_id(&object) != Some(&stream.creation_id) {
            return Err(side.reject(
                "stream_changed",
                &stream.id,
                "stream identity changed or is missing",
            ));
        }
        Ok(())
    }

    fn inlet(
        &self,
        snapshot: &FlowResultRecord,
        stream: &H3FlowStream,
        properties: &rf_model::equipment::ConcentricTubeLiquidProperties,
        side: Side,
    ) -> RfResult<ConcentricTubeLiquidInlet> {
        self.check_stream(stream, side)?;
        let record = snapshot
            .streams
            .iter()
            .find(|record| record.stream_id == stream.id)
            .ok_or_else(|| {
                side.reject(
                    "stream_missing",
                    &stream.id,
                    "stream absent from the current result",
                )
            })?;
        self.qualification
            .inlet(record, properties, self.saved.basis(), side)
    }
}

fn parse_project(contents: &str) -> RfResult<Box<StoredProjectV4>> {
    match project_v4::parse_with_equipment_support(contents, FLOW_SUPPORT, EQUIPMENT_SUPPORT)? {
        ProjectV4Read::Editable(project) => Ok(project),
        ProjectV4Read::ReadOnly(project) => Err(rejected(
            "unsupported_project",
            project.reasons().join(", "),
        )),
    }
}

#[derive(Clone, Copy)]
enum Side {
    Hot,
    Cold,
}

impl Side {
    fn reject(self, code: &str, stream: &StreamId, text: &str) -> RfError {
        let side = match self {
            Self::Hot => "hot",
            Self::Cold => "cold",
        };
        rejected(code, format!("{side} inlet {stream}: {text}"))
            .with_related_stream_id(stream.clone())
    }
}

fn rejected(code: &str, message: impl Into<String>) -> RfError {
    RfError::invalid_input(message).with_diagnostic_code(format!("equipment.h3.flow.{code}"))
}

fn inputs_changed() -> RfError {
    rejected(
        "inputs_changed",
        "the saved case or accepted flow result changed",
    )
}
