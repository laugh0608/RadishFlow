use rf_types::units::{
    ALL_QUANTITIES, DisplayUnitSet, DisplayUnitSetError, MeasurementUnit, QuantityKind,
};
use std::collections::BTreeMap;

/// Session identity supplied by the owner of an Inspector view; never persisted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayUnitViewId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewDisplayUnits {
    units: [Option<MeasurementUnit>; ALL_QUANTITIES.len()],
}
impl Default for ViewDisplayUnits {
    fn default() -> Self {
        Self {
            units: [None; ALL_QUANTITIES.len()],
        }
    }
}
impl ViewDisplayUnits {
    pub fn unit_for(&self, quantity: QuantityKind) -> Option<MeasurementUnit> {
        self.units[Self::index(quantity)]
    }
    pub fn set_unit(
        &mut self,
        quantity: QuantityKind,
        unit: Option<MeasurementUnit>,
    ) -> Result<(), DisplayUnitSetError> {
        if let Some(unit) = unit {
            DisplayUnitSet::si().set_unit(quantity, unit)?;
        }
        self.units[Self::index(quantity)] = unit;
        Ok(())
    }
    pub fn is_empty(&self) -> bool {
        self.units.iter().all(Option::is_none)
    }
    fn index(quantity: QuantityKind) -> usize {
        ALL_QUANTITIES
            .iter()
            .position(|candidate| *candidate == quantity)
            .expect("catalog covers all quantities")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PresentationChange {
    Project(DisplayUnitSet),
    View(DisplayUnitViewId, ViewDisplayUnits),
}

/// Presentation edits never enter engineering history or advance solve revisions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProjectPresentationState {
    current: DisplayUnitSet,
    saved: DisplayUnitSet,
    views: BTreeMap<DisplayUnitViewId, ViewDisplayUnits>,
    undo: Vec<PresentationChange>,
    redo: Vec<PresentationChange>,
    source_file_version: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectPresentationCommand {
    Apply(DisplayUnitSet),
    ApplyView {
        view: DisplayUnitViewId,
        units: ViewDisplayUnits,
    },
    CloseView(DisplayUnitViewId),
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

    pub fn view_units(&self, view: DisplayUnitViewId) -> ViewDisplayUnits {
        self.views.get(&view).cloned().unwrap_or_default()
    }

    pub fn effective_unit(
        &self,
        view: Option<DisplayUnitViewId>,
        quantity: QuantityKind,
    ) -> MeasurementUnit {
        view.and_then(|id| self.views.get(&id))
            .and_then(|units| units.unit_for(quantity))
            .unwrap_or_else(|| self.current.unit_for(quantity))
    }

    /// View retirement is not undoable. Remove only that view's history on both sides.
    pub fn close_view(&mut self, view: DisplayUnitViewId) -> bool {
        let removed = self.views.remove(&view).is_some();
        let previous = self.undo.len() + self.redo.len();
        self.undo
            .retain(|change| !matches!(change, PresentationChange::View(id, _) if *id == view));
        self.redo
            .retain(|change| !matches!(change, PresentationChange::View(id, _) if *id == view));
        removed || previous != self.undo.len() + self.redo.len()
    }

    pub fn apply(&mut self, command: ProjectPresentationCommand) -> bool {
        let change = match command {
            ProjectPresentationCommand::Apply(next) => PresentationChange::Project(next),
            ProjectPresentationCommand::ApplyView { view, units } => {
                PresentationChange::View(view, units)
            }
            ProjectPresentationCommand::CloseView(view) => return self.close_view(view),
            ProjectPresentationCommand::RestoreSaved => {
                PresentationChange::Project(self.saved.clone())
            }
            ProjectPresentationCommand::Undo => {
                let Some(previous) = self.undo.pop() else {
                    return false;
                };
                let inverse = self.replace(previous);
                self.redo.push(inverse);
                return true;
            }
            ProjectPresentationCommand::Redo => {
                let Some(next) = self.redo.pop() else {
                    return false;
                };
                let inverse = self.replace(next);
                self.undo.push(inverse);
                return true;
            }
        };
        let unchanged = match &change {
            PresentationChange::Project(next) => next == &self.current,
            PresentationChange::View(view, next) => *next == self.view_units(*view),
        };
        if unchanged {
            return false;
        }
        let inverse = self.replace(change);
        self.undo.push(inverse);
        self.redo.clear();
        true
    }

    fn replace(&mut self, change: PresentationChange) -> PresentationChange {
        match change {
            PresentationChange::Project(next) => {
                PresentationChange::Project(std::mem::replace(&mut self.current, next))
            }
            PresentationChange::View(view, next) => {
                let previous = self.view_units(view);
                if next.is_empty() {
                    self.views.remove(&view);
                } else {
                    self.views.insert(view, next);
                }
                PresentationChange::View(view, previous)
            }
        }
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

#[cfg(test)]
mod view_tests {
    use super::*;
    use MeasurementUnit as U;
    use QuantityKind as Q;
    fn temperature(unit: U) -> ViewDisplayUnits {
        let mut units = ViewDisplayUnits::default();
        units.set_unit(Q::AbsoluteTemperature, Some(unit)).unwrap();
        units
    }
    #[test]
    fn explicit_override_is_sparse_validated_and_not_a_project_save_change() {
        let mut state = ProjectPresentationState::from_loaded(DisplayUnitSet::si(), 2);
        let view = DisplayUnitViewId(1);
        let mut units = temperature(U::Celsius);
        let before = units.clone();
        assert!(
            units
                .set_unit(Q::AbsoluteTemperature, Some(U::Bar))
                .is_err()
        );
        assert_eq!(units, before);
        state.apply(ProjectPresentationCommand::ApplyView { view, units });
        assert!(!state.is_dirty());
        assert_eq!(
            state.effective_unit(Some(view), Q::AbsoluteTemperature),
            U::Celsius
        );
        assert_eq!(
            state.effective_unit(Some(view), Q::AbsolutePressure),
            U::Pascal
        );
        assert_eq!(
            state.effective_unit(Some(DisplayUnitViewId(2)), Q::AbsoluteTemperature),
            U::Kelvin
        );
        state.apply(ProjectPresentationCommand::Apply(
            DisplayUnitSet::engineering(),
        ));
        assert_eq!(
            state.effective_unit(Some(view), Q::AbsolutePressure),
            U::Bar
        );
        state.apply(ProjectPresentationCommand::ApplyView {
            view,
            units: ViewDisplayUnits::default(),
        });
        assert!(state.view_units(view).is_empty());
        state.apply(ProjectPresentationCommand::Undo);
        assert_eq!(
            state.view_units(view).unit_for(Q::AbsoluteTemperature),
            Some(U::Celsius)
        );
        assert!(!state.apply(ProjectPresentationCommand::ApplyView {
            view,
            units: temperature(U::Celsius)
        }));
        assert!(state.can_redo());
    }
    #[test]
    fn mixed_history_and_view_retirement_cannot_resurrect_closed_overrides() {
        let mut state = ProjectPresentationState::from_loaded(DisplayUnitSet::si(), 2);
        let a = DisplayUnitViewId(1);
        let b = DisplayUnitViewId(2);
        state.apply(ProjectPresentationCommand::ApplyView {
            view: a,
            units: temperature(U::Celsius),
        });
        state.apply(ProjectPresentationCommand::Apply(
            DisplayUnitSet::engineering(),
        ));
        state.apply(ProjectPresentationCommand::ApplyView {
            view: b,
            units: temperature(U::Kelvin),
        });
        state.record_saved(DisplayUnitSet::engineering(), 2);
        state.apply(ProjectPresentationCommand::Undo);
        assert!(!state.is_dirty());
        assert!(state.close_view(a)); // removes a's undo entries
        assert!(state.close_view(b)); // removes b's redo entries
        assert!(!state.can_redo());
        assert!(state.apply(ProjectPresentationCommand::Undo));
        assert_eq!(state.display_units(), &DisplayUnitSet::si());
        assert!(state.is_dirty());
        assert!(!state.can_undo());
        state.apply(ProjectPresentationCommand::Redo);
        assert!(!state.is_dirty());
        for view in [a, b] {
            assert!(state.view_units(view).is_empty());
        }
        let mut restored = state.clone();
        restored.apply(ProjectPresentationCommand::ApplyView {
            view: a,
            units: temperature(U::Kelvin),
        });
        restored.apply(ProjectPresentationCommand::RestoreSaved);
        assert_eq!(
            restored.effective_unit(Some(a), Q::AbsoluteTemperature),
            U::Kelvin
        );
    }
}
