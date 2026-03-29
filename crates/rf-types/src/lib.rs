use std::error::Error;
use std::fmt;

macro_rules! define_string_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

impl RfError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn code(&self) -> ErrorCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
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
