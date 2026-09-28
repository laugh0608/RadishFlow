use rf_types::units::DisplayUnitSet;

/// Presentation edits never enter engineering history or advance solve revisions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProjectPresentationState {
    current: DisplayUnitSet,
    saved: DisplayUnitSet,
    undo: Vec<DisplayUnitSet>,
    redo: Vec<DisplayUnitSet>,
    source_file_version: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectPresentationCommand {
    Apply(DisplayUnitSet),
    Undo,
    Redo,
    RestoreSaved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectSaveState {
    pub document_dirty: bool,
    pub presentation_dirty: bool,
    pub pending_input_count: usize,
}

impl ProjectSaveState {
    pub fn has_unsaved_changes(self) -> bool {
        self.document_dirty || self.presentation_dirty
    }

    pub fn needs_close_confirmation(self) -> bool {
        self.has_unsaved_changes() || self.pending_input_count > 0
    }
}

impl ProjectPresentationState {
    pub fn from_loaded(display_units: DisplayUnitSet, source_file_version: u32) -> Self {
        Self {
            current: display_units.clone(),
            saved: display_units,
            source_file_version: Some(source_file_version),
            ..Self::default()
        }
    }

    /// A new document adopts the caller's resolved default without creating edit history.
    pub fn for_new_project(display_units: DisplayUnitSet) -> Self {
        Self {
            current: display_units.clone(),
            saved: display_units,
            ..Self::default()
        }
    }

    pub fn display_units(&self) -> &DisplayUnitSet {
        &self.current
    }

    pub fn source_file_version(&self) -> Option<u32> {
        self.source_file_version
    }

    pub fn is_dirty(&self) -> bool {
        self.current != self.saved
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Called only after the captured configuration was successfully written.
    /// Saving must not clear edit history; undo across a save becomes dirty again.
    pub fn record_saved(&mut self, written: DisplayUnitSet, version: u32) {
        self.saved = written;
        self.source_file_version = Some(version);
    }

    pub fn apply(&mut self, command: ProjectPresentationCommand) -> bool {
        let next = match command {
            ProjectPresentationCommand::Apply(next) => next,
            ProjectPresentationCommand::RestoreSaved => self.saved.clone(),
            ProjectPresentationCommand::Undo => {
                let Some(previous) = self.undo.pop() else {
                    return false;
                };
                self.redo
                    .push(std::mem::replace(&mut self.current, previous));
                return true;
            }
            ProjectPresentationCommand::Redo => {
                let Some(next) = self.redo.pop() else {
                    return false;
                };
                self.undo.push(std::mem::replace(&mut self.current, next));
                return true;
            }
        };
        if next == self.current {
            return false;
        }
        self.undo.push(std::mem::replace(&mut self.current, next));
        self.redo.clear();
        true
    }
}

impl super::WorkspaceState {
    pub fn project_save_state(&self) -> ProjectSaveState {
        ProjectSaveState {
            document_dirty: self.last_saved_revision != Some(self.document.revision),
            presentation_dirty: self.project_presentation.is_dirty(),
            pending_input_count: self.drafts.pending_count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_history_survives_saving_and_noop_preserves_redo() {
        let mut state = ProjectPresentationState::from_loaded(DisplayUnitSet::si(), 1);
        assert!(state.apply(ProjectPresentationCommand::Apply(
            DisplayUnitSet::engineering()
        )));
        assert!(state.is_dirty());
        state.record_saved(DisplayUnitSet::engineering(), 2);
        assert!(!state.is_dirty());
        assert!(state.apply(ProjectPresentationCommand::Undo));
        assert!(state.is_dirty());
        assert!(!state.apply(ProjectPresentationCommand::Apply(DisplayUnitSet::si())));
        assert!(state.apply(ProjectPresentationCommand::Redo));
        assert!(!state.is_dirty());
        assert!(state.apply(ProjectPresentationCommand::Undo));
        assert!(state.apply(ProjectPresentationCommand::RestoreSaved));
        assert!(!state.is_dirty());
        assert!(!state.can_redo());
    }
}
