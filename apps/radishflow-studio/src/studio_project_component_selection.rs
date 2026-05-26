use std::time::SystemTime;

use rf_model::Component;
use rf_types::{RfError, RfResult};
use rf_ui::AppState;

pub const PROJECT_COMPONENT_SELECT_COMMAND_PREFIX: &str = "project.component.select:";
pub const PROJECT_COMPONENT_REMOVE_COMMAND_PREFIX: &str = "project.component.remove:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StudioBuiltinProjectComponent {
    pub component_id: &'static str,
    pub name: &'static str,
    pub formula: &'static str,
}

impl StudioBuiltinProjectComponent {
    pub fn to_model_component(self) -> Component {
        Component::new(self.component_id, self.name).with_formula(self.formula)
    }
}

pub const STUDIO_BUILTIN_PROJECT_COMPONENTS: &[StudioBuiltinProjectComponent] = &[
    StudioBuiltinProjectComponent {
        component_id: "methane",
        name: "Methane",
        formula: "CH4",
    },
    StudioBuiltinProjectComponent {
        component_id: "ethane",
        name: "Ethane",
        formula: "C2H6",
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioProjectComponentSelectionCommand {
    pub component_id: String,
}

impl StudioProjectComponentSelectionCommand {
    pub fn new(component_id: impl Into<String>) -> Self {
        Self {
            component_id: component_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectComponentSelectionOutcome {
    pub command: StudioProjectComponentSelectionCommand,
    pub applied: bool,
    pub selected_component_ids: Vec<String>,
    pub document_revision: u64,
    pub command_history_len: usize,
}

pub fn project_component_select_command_id(component_id: &str) -> String {
    format!("{PROJECT_COMPONENT_SELECT_COMMAND_PREFIX}{component_id}")
}

pub fn project_component_remove_command_id(component_id: &str) -> String {
    format!("{PROJECT_COMPONENT_REMOVE_COMMAND_PREFIX}{component_id}")
}

pub fn project_component_select_command_from_id(
    command_id: &str,
) -> Option<StudioProjectComponentSelectionCommand> {
    command_id
        .strip_prefix(PROJECT_COMPONENT_SELECT_COMMAND_PREFIX)
        .filter(|component_id| !component_id.is_empty())
        .map(StudioProjectComponentSelectionCommand::new)
}

pub fn project_component_remove_command_from_id(
    command_id: &str,
) -> Option<StudioProjectComponentSelectionCommand> {
    command_id
        .strip_prefix(PROJECT_COMPONENT_REMOVE_COMMAND_PREFIX)
        .filter(|component_id| !component_id.is_empty())
        .map(StudioProjectComponentSelectionCommand::new)
}

pub fn builtin_project_component(component_id: &str) -> Option<StudioBuiltinProjectComponent> {
    STUDIO_BUILTIN_PROJECT_COMPONENTS
        .iter()
        .copied()
        .find(|component| component.component_id == component_id)
}

pub fn select_project_component(
    app_state: &mut AppState,
    command: StudioProjectComponentSelectionCommand,
) -> RfResult<ProjectComponentSelectionOutcome> {
    select_project_component_at(app_state, command, SystemTime::now())
}

pub fn select_project_component_at(
    app_state: &mut AppState,
    command: StudioProjectComponentSelectionCommand,
    changed_at: rf_ui::DateTimeUtc,
) -> RfResult<ProjectComponentSelectionOutcome> {
    let component = builtin_project_component(&command.component_id)
        .ok_or_else(|| {
            RfError::invalid_input(format!(
                "component `{}` is not an enabled built-in Studio component",
                command.component_id
            ))
        })?
        .to_model_component();

    let applied = app_state
        .add_flowsheet_component(component, changed_at)?
        .is_some();

    Ok(project_component_selection_outcome(
        app_state, command, applied,
    ))
}

pub fn remove_project_component(
    app_state: &mut AppState,
    command: StudioProjectComponentSelectionCommand,
) -> RfResult<ProjectComponentSelectionOutcome> {
    remove_project_component_at(app_state, command, SystemTime::now())
}

pub fn remove_project_component_at(
    app_state: &mut AppState,
    command: StudioProjectComponentSelectionCommand,
    changed_at: rf_ui::DateTimeUtc,
) -> RfResult<ProjectComponentSelectionOutcome> {
    if builtin_project_component(&command.component_id).is_none() {
        return Err(RfError::invalid_input(format!(
            "component `{}` is not an enabled built-in Studio component",
            command.component_id
        )));
    }

    let applied = app_state
        .remove_flowsheet_component(command.component_id.clone().into(), changed_at)?
        .is_some();

    Ok(project_component_selection_outcome(
        app_state, command, applied,
    ))
}

fn project_component_selection_outcome(
    app_state: &AppState,
    command: StudioProjectComponentSelectionCommand,
    applied: bool,
) -> ProjectComponentSelectionOutcome {
    ProjectComponentSelectionOutcome {
        command,
        applied,
        selected_component_ids: app_state
            .workspace
            .document
            .flowsheet
            .components
            .keys()
            .map(|component_id| component_id.as_str().to_string())
            .collect(),
        document_revision: app_state.workspace.document.revision,
        command_history_len: app_state.workspace.command_history.len(),
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use rf_model::Flowsheet;
    use rf_ui::{AppState, DocumentCommand, DocumentMetadata, FlowsheetDocument};

    use super::{
        StudioProjectComponentSelectionCommand, project_component_remove_command_from_id,
        project_component_remove_command_id, project_component_select_command_from_id,
        project_component_select_command_id, remove_project_component_at,
        select_project_component_at,
    };

    #[test]
    fn select_project_component_writes_flowsheet_components() {
        let document = FlowsheetDocument::new(
            Flowsheet::new("demo"),
            DocumentMetadata::new("doc-1", "Demo", UNIX_EPOCH),
        );
        let mut app_state = AppState::new(document);

        let outcome = select_project_component_at(
            &mut app_state,
            StudioProjectComponentSelectionCommand::new("methane"),
            UNIX_EPOCH + Duration::from_secs(10),
        )
        .expect("expected built-in component selection");

        assert!(outcome.applied);
        assert_eq!(outcome.selected_component_ids, ["methane"]);
        assert_eq!(
            app_state
                .workspace
                .command_history
                .current_entry()
                .map(|entry| &entry.command),
            Some(&DocumentCommand::AddComponent {
                component_id: "methane".into(),
                name: "Methane".to_string(),
                formula: Some("CH4".to_string()),
            })
        );
    }

    #[test]
    fn remove_project_component_deletes_unused_flowsheet_component() {
        let mut flowsheet = Flowsheet::new("demo");
        flowsheet
            .insert_component(rf_model::Component::new("methane", "Methane").with_formula("CH4"))
            .expect("expected methane insert");
        let document = FlowsheetDocument::new(
            flowsheet,
            DocumentMetadata::new("doc-1", "Demo", UNIX_EPOCH),
        );
        let mut app_state = AppState::new(document);

        let outcome = remove_project_component_at(
            &mut app_state,
            StudioProjectComponentSelectionCommand::new("methane"),
            UNIX_EPOCH + Duration::from_secs(10),
        )
        .expect("expected built-in component removal");

        assert!(outcome.applied);
        assert!(outcome.selected_component_ids.is_empty());
        assert_eq!(
            app_state
                .workspace
                .command_history
                .current_entry()
                .map(|entry| &entry.command),
            Some(&DocumentCommand::RemoveComponent {
                component_id: "methane".into(),
            })
        );
    }

    #[test]
    fn project_component_command_ids_round_trip_component_id() {
        let select_command_id = project_component_select_command_id("methane");
        let remove_command_id = project_component_remove_command_id("ethane");

        let select_command = project_component_select_command_from_id(&select_command_id)
            .expect("expected select command");
        let remove_command = project_component_remove_command_from_id(&remove_command_id)
            .expect("expected remove command");

        assert_eq!(select_command.component_id, "methane");
        assert_eq!(remove_command.component_id, "ethane");
    }
}
