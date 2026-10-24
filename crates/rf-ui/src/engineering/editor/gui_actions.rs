//! GUI-owned draft submission shares formal actions without a force/bypass option.
use super::*;

impl EquipmentEditorState {
    pub fn dispatch_with_receipt(
        &mut self,
        session: &mut EngineeringSession,
        units: &DisplayUnitSet,
        command: EquipmentEditorCommand,
    ) -> Result<Option<EquipmentActionReceipt>, EquipmentEditError> {
        let selecting = matches!(command, EquipmentEditorCommand::Select(_));
        let target = command.error_target().or_else(|| self.active.clone());
        let result = self.dispatch_gui_action(session, units, command);
        if let Err(error) = &result {
            self.error = Some(error.clone());
            if let Some(target) = target {
                self.target_errors.insert(target, error.clone());
            }
        } else {
            self.error = None;
            if !selecting && let Some(target) = target {
                self.target_errors.remove(&target);
            }
        }
        result
    }

    fn dispatch_gui_action(
        &mut self,
        session: &mut EngineeringSession,
        units: &DisplayUnitSet,
        command: EquipmentEditorCommand,
    ) -> Result<Option<EquipmentActionReceipt>, EquipmentEditError> {
        use EquipmentEditorCommand::*;
        let action = match command {
            CreateTask => EquipmentAction::CreateSizingTask,
            CreateRatingTask => EquipmentAction::CreateRatingTask,
            CreateSource => EquipmentAction::CreateSource,
            AddRatingCase { task, copy_from } => EquipmentAction::AddRatingCase { task, copy_from },
            ReorderRatingCases { task, cases } => {
                EquipmentAction::ReorderRatingCases { task, cases }
            }
            Delete(target) => EquipmentAction::Delete(target),
            History { redo } => EquipmentAction::History { redo },
            Confirm(key) => EquipmentAction::Confirm(key),
            ConfirmGeometryAdoption(key) => EquipmentAction::ConfirmAdoption(key),
            Check(task) => EquipmentAction::CheckSizing(task),
            CheckRating(member) => EquipmentAction::CheckRating(member),
            Apply(_) | PreviewGeometryAdoption(_) => {
                let before = EquipmentActionContext::capture(session);
                let old_inputs = session.inputs().clone();
                let target = command.error_target();
                // This command validates the stored draft baseline and consumes the
                // currently owned draft, then calls the same submit_input transaction.
                // Stage proposal replacement too, so a failed draft parse preserves it.
                let mut editor = self.clone();
                let mut formal = session.clone();
                editor.pending = None;
                editor.adoption_preview = None;
                editor.execute(&mut formal, units, command)?;
                let target = target.or_else(|| {
                    editor.adoption_preview.as_ref().map(|p| {
                        EquipmentEditorTarget::RatingGeometry(
                            p.proposal().target().task_id().clone(),
                        )
                    })
                });
                let receipt = editor.action_receipt(&formal, before, &old_inputs, target, None);
                *self = editor;
                *session = formal;
                return Ok(Some(receipt));
            }
            Calculate(_) | CalculateRating(_) => {
                return Err(EquipmentEditError::UnsupportedEditorAction);
            }
            command => {
                self.dispatch(session, units, command)?;
                return Ok(None);
            }
        };
        self.commit_action(session, units, action).map(Some)
    }
}
