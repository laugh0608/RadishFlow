use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

macro_rules! define_string_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.into_inner()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

define_string_id!(ComponentId);
define_string_id!(StreamId);
define_string_id!(UnitId);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagnosticPortTarget {
    pub unit_id: UnitId,
    pub port_name: String,
}

impl DiagnosticPortTarget {
    pub fn new(unit_id: impl Into<UnitId>, port_name: impl Into<String>) -> Self {
        Self {
            unit_id: unit_id.into(),
            port_name: port_name.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PhaseLabel {
    Overall,
    Liquid,
    Vapor,
}

impl PhaseLabel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Overall => "overall",
            Self::Liquid => "liquid",
            Self::Vapor => "vapor",
        }
    }
}

impl fmt::Display for PhaseLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PortDirection {
    Inlet,
    Outlet,
}

impl PortDirection {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inlet => "inlet",
            Self::Outlet => "outlet",
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            Self::Inlet => Self::Outlet,
            Self::Outlet => Self::Inlet,
        }
    }
}

impl fmt::Display for PortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PortKind {
    Material,
    Energy,
    Information,
}

impl PortKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Material => "material",
            Self::Energy => "energy",
            Self::Information => "information",
        }
    }
}

impl fmt::Display for PortKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    InvalidInput,
    DuplicateId,
    MissingEntity,
    InvalidConnection,
    Thermo,
    Flash,
    NotImplemented,
}

impl ErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::DuplicateId => "duplicate_id",
            Self::MissingEntity => "missing_entity",
            Self::InvalidConnection => "invalid_connection",
            Self::Thermo => "thermo",
            Self::Flash => "flash",
            Self::NotImplemented => "not_implemented",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RfError {
    code: ErrorCode,
    message: String,
    context: RfErrorContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RfErrorContext {
    diagnostic_code: Option<String>,
    related_unit_ids: Vec<UnitId>,
    related_stream_ids: Vec<StreamId>,
    related_port_targets: Vec<DiagnosticPortTarget>,
}

impl RfErrorContext {
    pub fn diagnostic_code(&self) -> Option<&str> {
        self.diagnostic_code.as_deref()
    }

    pub fn related_unit_ids(&self) -> &[UnitId] {
        &self.related_unit_ids
    }

    pub fn related_stream_ids(&self) -> &[StreamId] {
        &self.related_stream_ids
    }

    pub fn related_port_targets(&self) -> &[DiagnosticPortTarget] {
        &self.related_port_targets
    }
}

impl RfError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            context: RfErrorContext::default(),
        }
    }

    pub fn code(&self) -> ErrorCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn context(&self) -> &RfErrorContext {
        &self.context
    }

    pub fn with_diagnostic_code(mut self, diagnostic_code: impl Into<String>) -> Self {
        self.context.diagnostic_code = Some(diagnostic_code.into());
        self
    }

    pub fn with_related_unit_id(mut self, unit_id: impl Into<UnitId>) -> Self {
        let unit_id = unit_id.into();
        if !self
            .context
            .related_unit_ids
            .iter()
            .any(|existing| existing == &unit_id)
        {
            self.context.related_unit_ids.push(unit_id);
        }
        self
    }

    pub fn with_related_unit_ids(mut self, related_unit_ids: Vec<UnitId>) -> Self {
        self.context.related_unit_ids.clear();
        for unit_id in related_unit_ids {
            if !self
                .context
                .related_unit_ids
                .iter()
                .any(|existing| existing == &unit_id)
            {
                self.context.related_unit_ids.push(unit_id);
            }
        }
        self
    }

    pub fn with_related_stream_id(mut self, stream_id: impl Into<StreamId>) -> Self {
        let stream_id = stream_id.into();
        if !self
            .context
            .related_stream_ids
            .iter()
            .any(|existing| existing == &stream_id)
        {
            self.context.related_stream_ids.push(stream_id);
        }
        self
    }

    pub fn with_related_stream_ids(mut self, related_stream_ids: Vec<StreamId>) -> Self {
        self.context.related_stream_ids.clear();
        for stream_id in related_stream_ids {
            if !self
                .context
                .related_stream_ids
                .iter()
                .any(|existing| existing == &stream_id)
            {
                self.context.related_stream_ids.push(stream_id);
            }
        }
        self
    }

    pub fn with_related_port_target(mut self, target: impl Into<DiagnosticPortTarget>) -> Self {
        let target = target.into();
        if !self
            .context
            .related_port_targets
            .iter()
            .any(|existing| existing == &target)
        {
            self.context.related_port_targets.push(target);
        }
        self
    }

    pub fn with_related_port_targets(mut self, targets: Vec<DiagnosticPortTarget>) -> Self {
        self.context.related_port_targets.clear();
        for target in targets {
            if !self
                .context
                .related_port_targets
                .iter()
                .any(|existing| existing == &target)
            {
                self.context.related_port_targets.push(target);
            }
        }
        self
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidInput, message)
    }

    pub fn duplicate_id(entity: &'static str, id: impl fmt::Display) -> Self {
        Self::new(
            ErrorCode::DuplicateId,
            format!("duplicate {entity} id `{id}`"),
        )
    }

    pub fn missing_entity(entity: &'static str, id: impl fmt::Display) -> Self {
        Self::new(ErrorCode::MissingEntity, format!("missing {entity} `{id}`"))
    }

    pub fn invalid_connection(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidConnection, message)
    }

    pub fn thermo(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Thermo, message)
    }

    pub fn flash(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Flash, message)
    }

    pub fn not_implemented(feature: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotImplemented, feature)
    }
}

impl fmt::Display for RfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl Error for RfError {}

pub type RfResult<T> = Result<T, RfError>;

#[cfg(test)]
mod tests {
    use super::{DiagnosticPortTarget, RfError, StreamId, UnitId};

    #[test]
    fn rf_error_can_carry_stable_diagnostic_context() {
        let error = RfError::invalid_input("solve failed")
            .with_diagnostic_code("solver.step.execution")
            .with_related_unit_id("heater-1")
            .with_related_unit_id("heater-1")
            .with_related_unit_id("flash-1")
            .with_related_stream_id("stream-feed")
            .with_related_stream_id("stream-feed")
            .with_related_stream_id("stream-heated")
            .with_related_port_target(DiagnosticPortTarget::new("heater-1", "inlet"))
            .with_related_port_target(DiagnosticPortTarget::new("heater-1", "inlet"))
            .with_related_port_target(DiagnosticPortTarget::new("flash-1", "outlet"));

        assert_eq!(
            error.context().diagnostic_code(),
            Some("solver.step.execution")
        );
        assert_eq!(
            error.context().related_unit_ids(),
            [UnitId::new("heater-1"), UnitId::new("flash-1")].as_slice()
        );
        assert_eq!(
            error.context().related_stream_ids(),
            [StreamId::new("stream-feed"), StreamId::new("stream-heated")].as_slice()
        );
        assert_eq!(
            error.context().related_port_targets(),
            [
                DiagnosticPortTarget::new("heater-1", "inlet"),
                DiagnosticPortTarget::new("flash-1", "outlet")
            ]
            .as_slice()
        );
    }
}
