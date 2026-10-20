//! Session-local, reviewable actions derived from committed inputs and existing local rules.
//! No GUI wiring, transport DTO, persistence, automatic acceptance, or solve is introduced here.

use crate::modeling_actions::MaterialConnectionTarget;
use crate::modeling_specifications::{
    ModelingCheckPhase, ModelingCheckSessionId, SpecificationProblemKey,
    SpecificationReportIdentity, SpecificationTarget,
};
use rf_types::{RfError, StreamId};
use rf_ui::composition_commands::{CompositionNormalizationError, CompositionNormalizationPreview};
use rf_ui::{CanvasSuggestedMaterialConnection, DocumentCommand, DocumentId};
use std::sync::Arc;

mod candidates;
mod session;
pub use session::ModelingRecommendationSession;

pub const RECOMMENDATION_RULE_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationRule {
    LocateInput,
    NormalizeComposition,
    CreateOutletStream,
    ConnectMaterialStream,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecommendationTarget {
    Navigation {
        problem: Box<SpecificationProblemKey>,
        target: SpecificationTarget,
    },
    Composition(StreamId),
    Connection(MaterialConnectionTarget),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecommendationKey {
    pub document_id: DocumentId,
    pub session: ModelingCheckSessionId,
    pub rule: RecommendationRule,
    pub rule_version: u32,
    pub target: RecommendationTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecommendedAction {
    Locate(SpecificationTarget),
    NormalizeComposition(CompositionNormalizationPreview),
    /// A Create binding contains every initial stream value that the UI must preview.
    /// These are template inputs, never claimed to be user-confirmed specifications.
    MaterialConnection(CanvasSuggestedMaterialConnection),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelingRecommendation {
    pub key: RecommendationKey,
    pub identity: SpecificationReportIdentity,
    pub related_problems: Vec<SpecificationProblemKey>,
    pub action: RecommendedAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationVisibility {
    Available,
    Ignored,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecommendationEntry {
    pub recommendation: ModelingRecommendation,
    pub visibility: RecommendationVisibility,
}

/// Issued only by prepare(). The private owner capability also rejects tokens from
/// another newly constructed session with an identical persisted ID and revision.
#[derive(Debug, Clone)]
pub struct RecommendationPreview {
    owner: Arc<()>,
    sequence: u64,
    recommendation: ModelingRecommendation,
}

impl RecommendationPreview {
    pub fn recommendation(&self) -> &ModelingRecommendation {
        &self.recommendation
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecommendationError {
    DifferentDocument,
    DifferentSession,
    StaleRevision { expected: u64, actual: u64 },
    PreviewExpired,
    PreviewChanged,
    Unavailable,
    Ambiguous,
    Ignored,
    NavigationOnly,
    PendingDrafts,
    PendingCanvasEdit,
    ChecksUnavailable(ModelingCheckPhase),
    Normalization(CompositionNormalizationError),
    Rejected(RfError),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecommendationReceipt {
    pub key: RecommendationKey,
    pub command: DocumentCommand,
    pub revision: u64,
    /// A failed recheck does not undo a committed transaction or make it retryable.
    pub check_phase: ModelingCheckPhase,
}

#[cfg(test)]
mod tests;
