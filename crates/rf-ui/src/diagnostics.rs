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
    pub primary_code: Option<String>,
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
            primary_code: None,
            primary_message: primary_message.into(),
            diagnostic_count: 1,
            related_unit_ids: Vec::new(),
        }
    }

    pub fn with_primary_code(mut self, primary_code: impl Into<String>) -> Self {
        self.primary_code = Some(primary_code.into());
        self
    }

    pub fn with_primary_code_from_message(mut self) -> Self {
        self.primary_code = prefixed_diagnostic_code(&self.primary_message);
        self
    }

    pub fn with_related_unit_ids(mut self, related_unit_ids: Vec<UnitId>) -> Self {
        self.related_unit_ids = related_unit_ids;
        self
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

fn prefixed_diagnostic_code(message: &str) -> Option<String> {
    message
        .split(": ")
        .find(|segment| is_stable_diagnostic_code(segment))
        .map(str::to_string)
}

fn is_stable_diagnostic_code(candidate: &str) -> bool {
    candidate.starts_with("solver.")
        && candidate.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'.' || byte == b'_'
        })
}
