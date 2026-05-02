#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioDocumentHistoryCommand {
    Undo,
    Redo,
}

impl StudioDocumentHistoryCommand {
    pub const fn command_id(self) -> &'static str {
        match self {
            Self::Undo => EDIT_UNDO_COMMAND_ID,
            Self::Redo => EDIT_REDO_COMMAND_ID,
        }
    }
}

pub const EDIT_UNDO_COMMAND_ID: &str = "edit.undo";
pub const EDIT_REDO_COMMAND_ID: &str = "edit.redo";

pub fn document_history_command_from_id(command_id: &str) -> Option<StudioDocumentHistoryCommand> {
    match command_id {
        EDIT_UNDO_COMMAND_ID => Some(StudioDocumentHistoryCommand::Undo),
        EDIT_REDO_COMMAND_ID => Some(StudioDocumentHistoryCommand::Redo),
        _ => None,
    }
}
