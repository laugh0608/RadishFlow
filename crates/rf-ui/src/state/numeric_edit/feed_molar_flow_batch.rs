use super::*;
use crate::variable_browser::{ObjectId, VariableField, VariableSection};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedFeedMolarFlowBatch {
    input: crate::FlowRunQualification,
    drafts: Vec<(VariableId, u64)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedMolarFlowBatchError {
    TooFewStreams,
    StaleCheck,
    InvalidDrafts(Vec<(VariableId, NumericEditError)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedMolarFlowBatchReceipt {
    pub revision: u64,
    pub changed_count: usize,
}

struct PreparedBatch {
    next: Flowsheet,
    values: Vec<(StreamId, f64)>,
}

impl WorkspaceState {
    pub fn supports_feed_molar_flow_batch_item(&self, variable: &VariableId) -> bool {
        let (ObjectId::Stream(stream), VariableSection::Inputs, VariableField::MolarFlow) =
            (&variable.object, variable.section, &variable.field)
        else {
            return false;
        };
        let Some(reference) = self.watch_reference(variable.clone()).ok() else {
            return false;
        };
        if !self
            .variable_watch_list
            .entries()
            .contains(&reference.to_string())
        {
            return false;
        }
        let mut sources = self.document.flowsheet.units.values().flat_map(|unit| {
            unit.ports.iter().filter_map(move |port| {
                (port.kind == PortKind::Material
                    && port.direction == PortDirection::Outlet
                    && port.stream_id.as_ref() == Some(stream))
                .then_some(unit)
            })
        });
        sources.next().is_some_and(|unit| {
            BuiltinUnitKind::parse(&unit.kind) == Some(BuiltinUnitKind::Feed)
                && sources.next().is_none()
        })
    }

    pub fn check_feed_molar_flow_batch(
        &self,
        variables: &[VariableId],
    ) -> Result<CheckedFeedMolarFlowBatch, FeedMolarFlowBatchError> {
        self.prepare_feed_molar_flow_batch(variables)?;
        Ok(CheckedFeedMolarFlowBatch {
            input: self.flow_run_qualification(),
            drafts: variables
                .iter()
                .map(|id| {
                    (
                        id.clone(),
                        self.numeric_edit(id).expect("validated draft").generation(),
                    )
                })
                .collect(),
        })
    }

    fn prepare_feed_molar_flow_batch(
        &self,
        variables: &[VariableId],
    ) -> Result<PreparedBatch, FeedMolarFlowBatchError> {
        if variables.len() < 2 {
            return Err(FeedMolarFlowBatchError::TooFewStreams);
        }
        let mut next = self.document.flowsheet.clone();
        let mut values = Vec::new();
        let mut invalid = Vec::new();
        let mut seen = BTreeSet::new();
        for variable in variables {
            let result = (|| {
                let ObjectId::Stream(stream) = &variable.object else {
                    return Err(NumericEditError::UnsupportedField);
                };
                if !seen.insert(stream) || !self.supports_feed_molar_flow_batch_item(variable) {
                    return Err(NumericEditError::UnsupportedField);
                }
                let value = self.validated_numeric_candidate(self.numeric_edit(variable)?)?;
                let previous = self
                    .document
                    .flowsheet
                    .streams
                    .get(stream)
                    .ok_or(NumericEditError::UnsupportedField)?
                    .total_molar_flow_mol_s;
                apply_stream_specification_value(
                    &mut next,
                    stream,
                    &StreamInspectorDraftField::TotalMolarFlowMolS,
                    &CommandValue::Number(value),
                )
                .map_err(NumericEditError::Rejected)?;
                if value != previous {
                    values.push((stream.clone(), value));
                }
                Ok(())
            })();
            if let Err(error) = result {
                invalid.push((variable.clone(), error));
            }
        }
        if invalid.is_empty() {
            Ok(PreparedBatch { next, values })
        } else {
            Err(FeedMolarFlowBatchError::InvalidDrafts(invalid))
        }
    }
}

#[cfg(test)]
mod tests;

impl CheckedFeedMolarFlowBatch {
    pub fn matches(&self, workspace: &WorkspaceState, variables: &[VariableId]) -> bool {
        self.input.same_input(&workspace.flow_run_qualification())
            && self.drafts.len() == variables.len()
            && self
                .drafts
                .iter()
                .zip(variables)
                .all(|((id, generation), requested)| {
                    id == requested
                        && workspace.supports_feed_molar_flow_batch_item(id)
                        && workspace
                            .numeric_edit(id)
                            .is_ok_and(|draft| draft.generation() == *generation)
                })
    }
}

impl AppState {
    pub fn apply_feed_molar_flow_batch(
        &mut self,
        checked: &CheckedFeedMolarFlowBatch,
        changed_at: DateTimeUtc,
    ) -> Result<FeedMolarFlowBatchReceipt, FeedMolarFlowBatchError> {
        let variables: Vec<_> = checked.drafts.iter().map(|(id, _)| id.clone()).collect();
        let workspace = &mut self.workspace;
        if !checked.matches(workspace, &variables) {
            return Err(FeedMolarFlowBatchError::StaleCheck);
        }
        let PreparedBatch { next, values } = workspace.prepare_feed_molar_flow_batch(&variables)?;
        let changed_count = values.len();
        // The same keys were checked as part of resolving the sessions above.
        for id in &variables {
            workspace
                .drafts
                .fields
                .remove(&transaction::field_key(id).expect("validated field"));
        }
        let commit = workspace.commit_prepared_input(
            DocumentCommand::SetFeedMolarFlows { values },
            next,
            changed_at,
        );
        self.refresh_run_panel_state();
        Ok(FeedMolarFlowBatchReceipt {
            revision: commit.revision,
            changed_count,
        })
    }
}
