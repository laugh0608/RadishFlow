use rf_types::UnitId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticSummary {
    pub document_revision: u64,
    pub highest_severity: DiagnosticSeverity,
    pub primary_message: String,
    pub diagnostic_count: usize,
    pub related_unit_ids: Vec<UnitId>,
}

impl DiagnosticSummary {
    pub fn new(
        document_revision: u64,
        highest_severity: DiagnosticSeverity,
        primary_message: impl Into<String>,
    ) -> Self {
        Self {
            document_revision,
            highest_severity,
            primary_message: primary_message.into(),
            diagnostic_count: 1,
            related_unit_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticSnapshot {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub related_unit_ids: Vec<UnitId>,
}

impl DiagnosticSnapshot {
    pub fn new(
        severity: DiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            code: code.into(),
            message: message.into(),
            related_unit_ids: Vec::new(),
        }
    }
}
