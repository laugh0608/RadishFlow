use super::*;
use crate::engineering::editor::EquipmentEditorTarget;
use crate::variable_browser::{ActionDescriptor, ActionExecution};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquipmentActionKind {
    Edit(EquipmentEditorTarget),
    Check(EquipmentCaseKey),
    Size(EquipmentTaskId),
    Rate(EquipmentCaseKey),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentActionDescriptor {
    pub kind: EquipmentActionKind,
    pub modifies_inputs: bool,
    pub undoable: bool,
    pub execution: ActionExecution,
    pub pending_drafts: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectActionDescriptor {
    Flow(ActionDescriptor),
    Equipment(EquipmentActionDescriptor),
}
impl ProjectQuery<'_> {
    /// Existing capabilities, not a new public command dispatcher or availability grant.
    /// The selected execution adapter describes calculation completion semantics;
    /// its runtime must still perform preflight, quota and authorization checks.
    pub fn actions(
        &self,
        reference: Option<&ProjectObjectRef>,
        calculation_execution: ActionExecution,
    ) -> Result<Vec<ProjectActionDescriptor>, ProjectQueryError> {
        let Some(reference) = reference else {
            return Ok(self
                .flow_variables()
                .actions_with_execution(None, calculation_execution)
                .expect("flow scope")
                .into_iter()
                .map(ProjectActionDescriptor::Flow)
                .collect());
        };
        let record = self.record(reference)?;
        let mut kinds = Vec::new();
        let pending = match (&reference.object, record) {
            (ProjectObjectId::Flow(id), _) => {
                return Ok(self
                    .flow_variables()
                    .actions_with_execution(Some(id), calculation_execution)
                    .expect("validated flow object")
                    .into_iter()
                    .map(ProjectActionDescriptor::Flow)
                    .collect());
            }
            (ProjectObjectId::Equipment(id), ProjectRecord::Equipment(t)) => {
                let target = if t.as_sizing().is_some() {
                    EquipmentEditorTarget::Task(id.clone())
                } else {
                    EquipmentEditorTarget::RatingTask(id.clone())
                };
                kinds.push(EquipmentActionKind::Edit(target));
                self.workspace
                    .equipment_editor
                    .pending_for_task(id, &self.workspace.engineering)
            }
            (ProjectObjectId::Case(member), ProjectRecord::SizingCase(_)) => {
                kinds.extend([
                    EquipmentActionKind::Edit(EquipmentEditorTarget::Task(member.task_id.clone())),
                    EquipmentActionKind::Check(member.clone()),
                    EquipmentActionKind::Size(member.task_id.clone()),
                ]);
                self.workspace
                    .equipment_editor
                    .pending_for_task(&member.task_id, &self.workspace.engineering)
            }
            (ProjectObjectId::Case(member), ProjectRecord::RatingCase(_)) => {
                kinds.extend([
                    EquipmentActionKind::Edit(EquipmentEditorTarget::RatingCase(member.clone())),
                    EquipmentActionKind::Check(member.clone()),
                    EquipmentActionKind::Rate(member.clone()),
                ]);
                self.workspace
                    .equipment_editor
                    .pending_for_case(member, &self.workspace.engineering)
            }
            (id, _) => {
                let target = match id {
                    ProjectObjectId::Geometry(id) => {
                        EquipmentEditorTarget::RatingGeometry(id.clone())
                    }
                    ProjectObjectId::Basis(id) => EquipmentEditorTarget::Basis(id.clone()),
                    ProjectObjectId::Source(id) => EquipmentEditorTarget::Source(id.clone()),
                    _ => unreachable!("validated record kind"),
                };
                let pending = self
                    .workspace
                    .equipment_editor
                    .draft(&target)
                    .is_some_and(|d| d.is_pending());
                kinds.push(EquipmentActionKind::Edit(target));
                pending
            }
        };
        Ok(kinds
            .into_iter()
            .map(|kind| {
                let modifies_inputs = matches!(kind, EquipmentActionKind::Edit(_));
                let execution = if matches!(
                    kind,
                    EquipmentActionKind::Size(_) | EquipmentActionKind::Rate(_)
                ) {
                    calculation_execution
                } else {
                    ActionExecution::Inline
                };
                ProjectActionDescriptor::Equipment(EquipmentActionDescriptor {
                    kind,
                    modifies_inputs,
                    undoable: modifies_inputs,
                    execution,
                    pending_drafts: pending,
                })
            })
            .collect())
    }
}
