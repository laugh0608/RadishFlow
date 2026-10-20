use super::*;

impl From<SeparatorSizingTask> for EquipmentTask {
    fn from(value: SeparatorSizingTask) -> Self {
        Self::SeparatorSizing(value)
    }
}
impl From<SeparatorRatingTask> for EquipmentTask {
    fn from(value: SeparatorRatingTask) -> Self {
        Self::SeparatorRating(value)
    }
}
impl EquipmentTask {
    pub fn id(&self) -> &EquipmentTaskId {
        match self {
            Self::SeparatorSizing(t) => &t.id,
            Self::SeparatorRating(t) => &t.id,
        }
    }
    pub fn revision(&self) -> u64 {
        match self {
            Self::SeparatorSizing(t) => t.revision,
            Self::SeparatorRating(t) => t.revision,
        }
    }
    pub fn set_revision(&mut self, revision: u64) {
        match self {
            Self::SeparatorSizing(t) => t.revision = revision,
            Self::SeparatorRating(t) => t.revision = revision,
        }
    }
    pub fn tag(&self) -> &str {
        match self {
            Self::SeparatorSizing(t) => &t.tag,
            Self::SeparatorRating(t) => &t.tag,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Self::SeparatorSizing(t) => &t.name,
            Self::SeparatorRating(t) => &t.name,
        }
    }
    pub fn method(&self) -> &EquipmentMethod {
        match self {
            Self::SeparatorSizing(t) => &t.method,
            Self::SeparatorRating(t) => &t.method,
        }
    }
    pub fn as_sizing(&self) -> Option<&SeparatorSizingTask> {
        match self {
            Self::SeparatorSizing(t) => Some(t),
            _ => None,
        }
    }
    pub fn as_rating(&self) -> Option<&SeparatorRatingTask> {
        match self {
            Self::SeparatorRating(t) => Some(t),
            _ => None,
        }
    }
    pub fn case_ids(&self) -> impl Iterator<Item = &EngineeringCaseId> {
        let (sizing, rating) = match self {
            Self::SeparatorSizing(t) => (Some(&t.case.id), &[][..]),
            Self::SeparatorRating(t) => (None, t.cases.as_slice()),
        };
        sizing.into_iter().chain(rating.iter().map(|c| &c.id))
    }
}
