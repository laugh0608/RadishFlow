use std::time::SystemTime;

use rf_types::{RfError, RfResult};
use rf_ui::AppState;

pub const PROPERTY_PACKAGE_SELECT_COMMAND_PREFIX: &str = "project.property_package.select:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StudioBuiltinPropertyPackage {
    pub package_id: &'static str,
    pub label: &'static str,
    pub detail: &'static str,
    pub component_summary: &'static str,
}

pub const STUDIO_BUILTIN_PROPERTY_PACKAGES: &[StudioBuiltinPropertyPackage] =
    &[StudioBuiltinPropertyPackage {
        package_id: "binary-hydrocarbon-lite-v1",
        label: "Binary Hydrocarbon Lite",
        detail: "Local built-in methane/ethane package for MVP beta authoring.",
        component_summary: "Methane, Ethane",
    }];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioPropertyPackageSelectionCommand {
    pub package_id: String,
}

impl StudioPropertyPackageSelectionCommand {
    pub fn new(package_id: impl Into<String>) -> Self {
        Self {
            package_id: package_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyPackageSelectionOutcome {
    pub command: StudioPropertyPackageSelectionCommand,
    pub applied: bool,
    pub selected_package_id: Option<String>,
    pub document_revision: u64,
    pub command_history_len: usize,
}

pub fn property_package_select_command_id(package_id: &str) -> String {
    format!("{PROPERTY_PACKAGE_SELECT_COMMAND_PREFIX}{package_id}")
}

pub fn property_package_select_command_from_id(
    command_id: &str,
) -> Option<StudioPropertyPackageSelectionCommand> {
    command_id
        .strip_prefix(PROPERTY_PACKAGE_SELECT_COMMAND_PREFIX)
        .map(StudioPropertyPackageSelectionCommand::new)
}

pub fn builtin_property_package(package_id: &str) -> Option<StudioBuiltinPropertyPackage> {
    STUDIO_BUILTIN_PROPERTY_PACKAGES
        .iter()
        .copied()
        .find(|package| package.package_id == package_id)
}

pub fn select_property_package(
    app_state: &mut AppState,
    command: StudioPropertyPackageSelectionCommand,
) -> RfResult<PropertyPackageSelectionOutcome> {
    select_property_package_at(app_state, command, SystemTime::now())
}

pub fn select_property_package_at(
    app_state: &mut AppState,
    command: StudioPropertyPackageSelectionCommand,
    changed_at: rf_ui::DateTimeUtc,
) -> RfResult<PropertyPackageSelectionOutcome> {
    if builtin_property_package(&command.package_id).is_none() {
        return Err(RfError::invalid_input(format!(
            "property package `{}` is not an enabled built-in Studio package",
            command.package_id
        )));
    }

    let applied = app_state
        .set_flowsheet_property_package_id(Some(command.package_id.clone()), changed_at)?
        .is_some();

    Ok(PropertyPackageSelectionOutcome {
        command,
        applied,
        selected_package_id: app_state
            .workspace
            .document
            .flowsheet
            .property_package_id()
            .map(|package_id| package_id.to_string()),
        document_revision: app_state.workspace.document.revision,
        command_history_len: app_state.workspace.command_history.len(),
    })
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use rf_model::Flowsheet;
    use rf_ui::{AppState, DocumentCommand, DocumentMetadata, FlowsheetDocument};

    use super::{
        StudioPropertyPackageSelectionCommand, property_package_select_command_from_id,
        property_package_select_command_id, select_property_package_at,
    };

    #[test]
    fn select_property_package_writes_flowsheet_thermo_config() {
        let document = FlowsheetDocument::new(
            Flowsheet::new("demo"),
            DocumentMetadata::new("doc-1", "Demo", UNIX_EPOCH),
        );
        let mut app_state = AppState::new(document);

        let outcome = select_property_package_at(
            &mut app_state,
            StudioPropertyPackageSelectionCommand::new("binary-hydrocarbon-lite-v1"),
            UNIX_EPOCH + Duration::from_secs(10),
        )
        .expect("expected built-in package selection");

        assert!(outcome.applied);
        assert_eq!(
            app_state.workspace.document.flowsheet.property_package_id(),
            Some("binary-hydrocarbon-lite-v1")
        );
        assert_eq!(
            app_state
                .workspace
                .command_history
                .current_entry()
                .map(|entry| &entry.command),
            Some(&DocumentCommand::SetPropertyPackage {
                package_id: Some("binary-hydrocarbon-lite-v1".to_string())
            })
        );
    }

    #[test]
    fn property_package_select_command_round_trips_package_id() {
        let command_id = property_package_select_command_id("binary-hydrocarbon-lite-v1");
        let command = property_package_select_command_from_id(&command_id)
            .expect("expected property package command");

        assert_eq!(command.package_id, "binary-hydrocarbon-lite-v1");
    }
}
