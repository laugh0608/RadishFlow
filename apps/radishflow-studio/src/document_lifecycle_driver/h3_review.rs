//! Immutable saved input for the Studio review surface. It has no editable host.

use super::{StudioProjectOpen, StudioReadOnlyProject};
use rf_model::{engineering::EngineeringBasis, equipment::ConcentricTubeRatingTask};
use rf_store::{
    StoredDocumentMetadata, StoredProjectDocument, project_v3::StoredEngineeringInputs, sha256_hex,
};
use rf_types::units::DisplayUnitSet;
use rf_ui::engineering::{
    EquipmentCaseKey,
    concentric_tube::{ConcentricTubeRating, RatingRevisions, SavedRatingError, rate_saved_case},
};

#[derive(Debug)]
pub struct StudioH3Review {
    pub(super) original_json: String,
    pub(super) schema_version: u32,
    pub(super) revision: u64,
    metadata: StoredDocumentMetadata,
    sha256: String,
    units: DisplayUnitSet,
    inputs: StoredEngineeringInputs,
}

pub(super) fn open(
    document: &StoredProjectDocument,
    units: &DisplayUnitSet,
    schema_version: u32,
    engineering: &mut Option<StoredEngineeringInputs>,
    contents: &str,
) -> Option<StudioProjectOpen> {
    let view = StudioH3Review::take(document, units, schema_version, engineering, contents)?;
    Some(StudioProjectOpen::ReadOnly(Box::new(
        StudioReadOnlyProject::H3(Box::new(view)),
    )))
}

impl StudioH3Review {
    pub(super) fn take(
        document: &StoredProjectDocument,
        units: &DisplayUnitSet,
        schema_version: u32,
        engineering: &mut Option<StoredEngineeringInputs>,
        contents: &str,
    ) -> Option<Self> {
        if !engineering.as_ref()?.has_concentric_tube_rating() {
            return None;
        }
        Some(Self {
            original_json: contents.into(),
            schema_version,
            revision: document.revision,
            metadata: document.metadata.clone(),
            sha256: sha256_hex(contents.as_bytes()),
            units: units.clone(),
            inputs: engineering.take()?,
        })
    }

    pub fn metadata(&self) -> &StoredDocumentMetadata {
        &self.metadata
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn file_sha256(&self) -> &str {
        &self.sha256
    }

    pub fn display_units(&self) -> &DisplayUnitSet {
        &self.units
    }

    pub fn basis(&self) -> &EngineeringBasis {
        &self.inputs.basis
    }

    pub(super) fn task_count(&self) -> usize {
        self.inputs.tasks.len()
    }

    pub fn cases(&self) -> impl Iterator<Item = &ConcentricTubeRatingTask> {
        self.inputs
            .tasks
            .iter()
            .filter_map(|task| task.as_concentric_tube_rating())
    }

    pub fn independent_cases(&self) -> impl Iterator<Item = &ConcentricTubeRatingTask> {
        self.cases()
            .filter(|task| task.case.flow_association.is_none())
    }

    pub fn case(&self, member: &EquipmentCaseKey) -> Option<&ConcentricTubeRatingTask> {
        self.cases()
            .find(|task| task.id == member.task_id && task.case.id == member.case_id)
    }

    pub fn rate(
        &self,
        member: &EquipmentCaseKey,
        expected: RatingRevisions,
    ) -> Option<Result<ConcentricTubeRating, SavedRatingError>> {
        Some(rate_saved_case(self.case(member)?, self.basis(), expected))
    }
}
