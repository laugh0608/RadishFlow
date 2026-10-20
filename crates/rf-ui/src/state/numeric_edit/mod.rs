mod inspector;
mod parse;
mod presentation;
pub use presentation::*;
mod transaction;
pub use parse::*;
pub use transaction::NumericEditReceipt;

use super::*;
use crate::variable_browser::{BrowseError, VariableId};
use rf_types::units::{ConversionError, MeasurementUnit, QuantityKind, from_canonical};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumericEditError {
    Lookup(BrowseError),
    UnsupportedField,
    MissingSession,
    StaleGeneration { expected: u64, actual: u64 },
    FieldConflict,
    Incomplete,
    Invalid(NumericParseError),
    Rejected(RfError),
    Conversion(ConversionError),
}
impl std::fmt::Display for NumericEditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lookup(e) => write!(f, "numeric field lookup failed: {e:?}"),
            Self::UnsupportedField => f.write_str("field does not support numeric editing"),
            Self::MissingSession => f.write_str("numeric edit session no longer exists"),
            Self::StaleGeneration { expected, actual } => write!(
                f,
                "stale edit generation {expected}; current generation is {actual}"
            ),
            Self::FieldConflict => f.write_str(
                "committed value or source changed; explicitly restart or rebase the edit",
            ),
            Self::Incomplete => f.write_str("finish the numeric input before applying"),
            Self::Invalid(e) => write!(f, "invalid numeric input: {e:?}"),
            Self::Rejected(e) => write!(f, "numeric input rejected: {e}"),
            Self::Conversion(e) => write!(f, "{e}"),
        }
    }
}
impl std::error::Error for NumericEditError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericPrecisionSource {
    OriginalSi,
    ParsedText,
}

#[derive(Debug, Clone, PartialEq)]
struct EditContent {
    raw: String,
    unit: MeasurementUnit,
    origin: InputUnitOrigin,
    parsed: NumericParseOutcome,
    precision: NumericPrecisionSource,
    pending: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct FieldBaseline {
    value: Option<f64>,
    explicit: bool,
    inherited_stream: Option<StreamId>,
}

/// Sole owner of numeric text, input units and precision; domain validation is never historic.
#[derive(Debug, Clone, PartialEq)]
pub struct NumericEditSession {
    variable: VariableId,
    quantity: QuantityKind,
    generation: u64,
    baseline_revision: u64,
    baseline: FieldBaseline,
    current: EditContent,
    undo: Vec<EditContent>,
    redo: Vec<EditContent>,
    validation: Result<(), NumericEditError>,
    text_group: Option<(u64, u64)>,
    composition_original: Option<EditContent>,
}
impl NumericEditSession {
    pub fn variable(&self) -> &VariableId {
        &self.variable
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn raw_text(&self) -> &str {
        &self.current.raw
    }
    pub fn quantity(&self) -> QuantityKind {
        self.quantity
    }
    pub fn input_unit(&self) -> MeasurementUnit {
        self.current.unit
    }
    pub fn input_origin(&self) -> InputUnitOrigin {
        self.current.origin
    }
    pub fn precision_source(&self) -> NumericPrecisionSource {
        self.current.precision
    }
    pub fn parse_outcome(&self) -> &NumericParseOutcome {
        &self.current.parsed
    }
    pub fn validation(&self) -> &Result<(), NumericEditError> {
        &self.validation
    }
    pub fn is_dirty(&self) -> bool {
        match self.parsed_si() {
            Ok(value) => self.baseline.value != Some(value) || !self.baseline.explicit,
            Err(_) => self.current.pending,
        }
    }
    pub fn is_pending(&self) -> bool {
        self.current.pending
    }
    pub fn draft_validation(&self) -> DraftValidationState {
        match &self.validation {
            Ok(()) => DraftValidationState::Valid,
            Err(NumericEditError::Incomplete) => DraftValidationState::Unknown,
            Err(_) => DraftValidationState::Invalid,
        }
    }
    pub fn candidate_si(&self) -> Result<f64, NumericEditError> {
        self.validation.clone()?;
        self.parsed_si()
    }
    fn parsed_si(&self) -> Result<f64, NumericEditError> {
        match &self.current.parsed {
            NumericParseOutcome::Value { si, .. } => Ok(*si),
            NumericParseOutcome::Incomplete => Err(NumericEditError::Incomplete),
            NumericParseOutcome::Invalid(e) => Err(NumericEditError::Invalid(e.clone())),
        }
    }
    fn replace(&mut self, next: EditContent) {
        if next != self.current {
            self.undo.push(std::mem::replace(&mut self.current, next));
            self.redo.clear();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumericEditEvent {
    /// One atomic edit. Native typing coalescing and IME boundaries belong to the consumer.
    ReplaceText(String),
    /// The consumer supplies a stable widget/session group only for contiguous plain typing.
    ReplaceTextGrouped {
        raw: String,
        group: (u64, u64),
    },
    PreviewComposition(String),
    CommitComposition(String),
    CancelComposition,
    SelectInputUnit(MeasurementUnit),
    Undo,
    Redo,
    /// Explicit conflict resolution, preserving the user's candidate and revalidating it.
    Rebase,
    AdoptValue,
}

#[cfg(test)]
mod tests;
