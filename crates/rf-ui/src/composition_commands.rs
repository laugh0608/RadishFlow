//! Explicit normalization of committed composition, independent of Inspector focus or drafts.

use rf_model::Composition;
use rf_types::{RfError, StreamId};
use rf_unitops::specifications::{CompositionCheck, CompositionPolicy, check_composition};

use crate::{
    AppState, CommandValue, DateTimeUtc, DocumentCommand, DocumentId, FlowsheetDocument,
    StreamInspectorDraftField, StreamSpecificationValue,
};

#[derive(Debug, Clone, PartialEq)]
pub struct CompositionNormalizationPreview {
    pub stream_id: StreamId,
    pub original: Composition,
    pub normalized: Composition,
    pub sum: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompositionNormalizationError {
    DifferentDocument,
    RevisionConflict { expected: u64, actual: u64 },
    PendingDrafts,
    PendingCanvasEdit,
    NotApplicable(CompositionCheck),
    PreviewChanged,
    Rejected(RfError),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompositionNormalizationRequest {
    pub document_id: DocumentId,
    pub expected_revision: u64,
    pub preview: CompositionNormalizationPreview,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompositionNormalizationReceipt {
    pub command: DocumentCommand,
    pub revision: u64,
}

/// Shared arithmetic; callers retain their own validation policy. In particular,
/// the legacy Inspector operation may normalize stored weights above one.
pub(crate) fn normalized_composition_values(composition: &Composition, sum: f64) -> Composition {
    composition
        .iter()
        .map(|(component, value)| (component.clone(), value / sum))
        .collect()
}

pub fn preview_composition_normalization(
    document: &FlowsheetDocument,
    stream_id: &StreamId,
) -> Result<CompositionNormalizationPreview, CompositionNormalizationError> {
    use CompositionNormalizationError::*;
    let stream = document.flowsheet.stream(stream_id).map_err(Rejected)?;
    let original = &stream.overall_mole_fractions;
    for component in original.keys() {
        if !document.flowsheet.components.contains_key(component) {
            return Err(Rejected(RfError::missing_entity(
                "project component",
                component,
            )));
        }
    }
    let check = check_composition(original, CompositionPolicy::SpecifiedMoleFractions);
    let CompositionCheck::Unnormalized { sum } = check else {
        return Err(NotApplicable(check));
    };
    let normalized = normalized_composition_values(original, sum);
    Ok(CompositionNormalizationPreview {
        stream_id: stream_id.clone(),
        original: original.clone(),
        normalized,
        sum,
    })
}

impl AppState {
    /// No selection, draft consumption, save, solve, or partial component commits.
    pub fn normalize_committed_stream_composition(
        &mut self,
        request: CompositionNormalizationRequest,
        changed_at: DateTimeUtc,
    ) -> Result<CompositionNormalizationReceipt, CompositionNormalizationError> {
        use CompositionNormalizationError::*;
        if request.document_id != self.workspace.document.metadata.document_id {
            return Err(DifferentDocument);
        }
        let actual = self.workspace.document.revision;
        if request.expected_revision != actual {
            return Err(RevisionConflict {
                expected: request.expected_revision,
                actual,
            });
        }
        if self.workspace.drafts.pending_count() > 0 {
            return Err(PendingDrafts);
        }
        if self.workspace.canvas_interaction.pending_edit.is_some() {
            return Err(PendingCanvasEdit);
        }
        let current = preview_composition_normalization(
            &self.workspace.document,
            &request.preview.stream_id,
        )?;
        if current != request.preview {
            return Err(PreviewChanged);
        }
        let command = DocumentCommand::SetStreamSpecifications {
            stream_id: current.stream_id,
            values: current
                .normalized
                .into_iter()
                .map(|(component, value)| StreamSpecificationValue {
                    field: StreamInspectorDraftField::OverallMoleFraction(component)
                        .command_field(),
                    value: CommandValue::Number(value),
                })
                .collect(),
        };
        let committed = self
            .workspace
            .commit_input_document_command(command.clone(), changed_at)
            .map_err(Rejected)?;
        self.refresh_run_panel_state();
        Ok(CompositionNormalizationReceipt {
            command,
            revision: committed.revision,
        })
    }
}
