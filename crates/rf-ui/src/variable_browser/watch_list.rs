use super::{StableVariableReference, VariableId, VariableReferenceError};
use crate::WorkspaceState;
use rf_types::{RfError, RfResult};

/// Project-owned order and saved baseline. Resolution is a projection of the workspace.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct VariableWatchList {
    entries: Vec<String>,
    saved: Option<Vec<String>>,
    generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableWatchCommand {
    Add(VariableId),
    Move {
        reference: String,
        direction: VariableWatchDirection,
    },
    Remove(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableWatchDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableWatchOutcome {
    Added,
    AlreadyWatched { reference: String },
    Moved { position: usize },
    Removed { next: Option<String> },
    Unchanged,
}

impl VariableWatchList {
    pub fn from_saved(saved: Option<Vec<String>>) -> Self {
        Self {
            entries: saved.as_deref().unwrap_or_default().to_vec(),
            saved,
            generation: 0,
        }
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    pub fn is_dirty(&self) -> bool {
        self.entries != self.saved.as_deref().unwrap_or_default()
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// An already declared capability remains present after the last item is removed.
    pub fn entries_for_save(&self) -> Option<&[String]> {
        (self.saved.is_some() || !self.entries.is_empty()).then_some(&self.entries)
    }

    pub fn record_saved(&mut self) {
        self.saved = self.entries_for_save().map(<[String]>::to_vec);
    }

    fn advance(&mut self) -> RfResult<()> {
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| RfError::invalid_input("watch list generation exhausted"))?;
        Ok(())
    }

    fn add(&mut self, reference: String) -> RfResult<VariableWatchOutcome> {
        if self.entries.contains(&reference) {
            return Ok(VariableWatchOutcome::AlreadyWatched { reference });
        }
        self.advance()?;
        self.entries.push(reference);
        Ok(VariableWatchOutcome::Added)
    }

    fn move_entry(
        &mut self,
        reference: &str,
        direction: VariableWatchDirection,
    ) -> RfResult<VariableWatchOutcome> {
        let index = self.index(reference)?;
        let destination = match direction {
            VariableWatchDirection::Up => index.checked_sub(1),
            VariableWatchDirection::Down => (index + 1 < self.entries.len()).then_some(index + 1),
        };
        let Some(destination) = destination else {
            return Ok(VariableWatchOutcome::Unchanged);
        };
        self.advance()?;
        self.entries.swap(index, destination);
        Ok(VariableWatchOutcome::Moved {
            position: destination + 1,
        })
    }

    fn remove(&mut self, reference: &str) -> RfResult<VariableWatchOutcome> {
        let index = self.index(reference)?;
        self.advance()?;
        self.entries.remove(index);
        let next = self
            .entries
            .get(index)
            .or_else(|| self.entries.last())
            .cloned();
        Ok(VariableWatchOutcome::Removed { next })
    }

    fn index(&self, reference: &str) -> RfResult<usize> {
        self.entries
            .iter()
            .position(|entry| entry == reference)
            .ok_or_else(|| {
                RfError::invalid_input("the watched reference is no longer in this project")
            })
    }
}

impl WorkspaceState {
    pub fn watch_reference(
        &self,
        variable: VariableId,
    ) -> Result<StableVariableReference, VariableReferenceError> {
        StableVariableReference::for_current_variable(self, variable)
    }

    pub fn edit_variable_watch_list(
        &mut self,
        command: &VariableWatchCommand,
    ) -> RfResult<VariableWatchOutcome> {
        match command {
            VariableWatchCommand::Add(variable) => {
                let reference = self.watch_reference(variable.clone()).map_err(|error| {
                    RfError::invalid_input(format!("cannot watch variable: {error:?}"))
                        .with_diagnostic_code("project.variable_watch_list.reference_rejected")
                })?;
                self.variable_watch_list.add(reference.to_string())
            }
            VariableWatchCommand::Move {
                reference,
                direction,
            } => self.variable_watch_list.move_entry(reference, *direction),
            VariableWatchCommand::Remove(reference) => self.variable_watch_list.remove(reference),
        }
    }
}
