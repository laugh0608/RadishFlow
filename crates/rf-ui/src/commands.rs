use rf_types::{StreamId, UnitId};

#[derive(Debug, Clone, PartialEq)]
pub enum CommandValue {
    Number(f64),
    Text(String),
    Bool(bool),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasPoint {
    pub x: f64,
    pub y: f64,
}

impl CanvasPoint {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentCommand {
    CreateUnit {
        unit_id: UnitId,
        kind: String,
    },
    DeleteUnit {
        unit_id: UnitId,
    },
    MoveUnit {
        unit_id: UnitId,
        position: CanvasPoint,
    },
    ConnectPorts {
        stream_id: StreamId,
        from_unit_id: UnitId,
        from_port: String,
        to_unit_id: Option<UnitId>,
        to_port: Option<String>,
    },
    DisconnectPorts {
        unit_id: UnitId,
        port: String,
    },
    RenameUnit {
        unit_id: UnitId,
        new_name: String,
    },
    SetUnitParameter {
        unit_id: UnitId,
        parameter: String,
        value: CommandValue,
    },
    SetStreamSpecification {
        stream_id: StreamId,
        field: String,
        value: CommandValue,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandHistoryEntry {
    pub revision: u64,
    pub command: DocumentCommand,
}

impl CommandHistoryEntry {
    pub fn new(revision: u64, command: DocumentCommand) -> Self {
        Self { revision, command }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CommandHistory {
    pub entries: Vec<CommandHistoryEntry>,
    pub cursor: usize,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    pub fn can_redo(&self) -> bool {
        self.cursor < self.entries.len()
    }

    pub fn current_entry(&self) -> Option<&CommandHistoryEntry> {
        self.cursor
            .checked_sub(1)
            .and_then(|index| self.entries.get(index))
    }

    pub fn record(&mut self, entry: CommandHistoryEntry) {
        if self.cursor < self.entries.len() {
            self.entries.truncate(self.cursor);
        }

        self.entries.push(entry);
        self.cursor = self.entries.len();
    }

    pub fn undo(&mut self) -> Option<CommandHistoryEntry> {
        if !self.can_undo() {
            return None;
        }

        self.cursor -= 1;
        self.entries.get(self.cursor).cloned()
    }

    pub fn redo(&mut self) -> Option<CommandHistoryEntry> {
        if !self.can_redo() {
            return None;
        }

        let entry = self.entries.get(self.cursor).cloned();
        self.cursor += 1;
        entry
    }
}
