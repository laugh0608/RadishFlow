use rf_model::Flowsheet;
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
    DeleteStream {
        stream_id: StreamId,
    },
    DisconnectPorts {
        unit_id: UnitId,
        port: String,
    },
    DisconnectPortAndDeleteStream {
        unit_id: UnitId,
        port: String,
        stream_id: StreamId,
    },
    RestoreCanonicalUnitPorts {
        unit_id: UnitId,
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
    SetStreamSpecifications {
        stream_id: StreamId,
        values: Vec<StreamSpecificationValue>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamSpecificationValue {
    pub field: String,
    pub value: CommandValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandHistoryEntry {
    pub revision: u64,
    pub command: DocumentCommand,
    pub before: Option<Flowsheet>,
    pub after: Option<Flowsheet>,
}

impl CommandHistoryEntry {
    pub fn new(revision: u64, command: DocumentCommand) -> Self {
        Self {
            revision,
            command,
            before: None,
            after: None,
        }
    }

    pub fn with_snapshots(
        revision: u64,
        command: DocumentCommand,
        before: Flowsheet,
        after: Flowsheet,
    ) -> Self {
        Self {
            revision,
            command,
            before: Some(before),
            after: Some(after),
        }
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

    pub fn undo_entry(&self) -> Option<&CommandHistoryEntry> {
        if !self.can_undo() {
            return None;
        }

        self.entries.get(self.cursor - 1)
    }

    pub fn redo_entry(&self) -> Option<&CommandHistoryEntry> {
        if !self.can_redo() {
            return None;
        }

        self.entries.get(self.cursor)
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

    pub fn step_undo(&mut self) -> Option<&CommandHistoryEntry> {
        if !self.can_undo() {
            return None;
        }

        self.cursor -= 1;
        self.entries.get(self.cursor)
    }

    pub fn step_redo(&mut self) -> Option<&CommandHistoryEntry> {
        if !self.can_redo() {
            return None;
        }

        let index = self.cursor;
        self.cursor += 1;
        self.entries.get(index)
    }
}
