use std::fmt;

/// Recoverable errors shared by semantic, storage, and test-harness code.
#[derive(Debug)]
pub enum Error {
    InvalidInput(String),
    InvalidRequest(String),
    Overloaded(String),
    ResponseTooLarge(String),
    Conflict(crate::transaction::TransactionConflict),
    Corruption(String),
    Io(std::io::Error),
    UnsupportedFormat(String),
    DurabilityFailure(String),
    RecoveryFailure(String),
    CheckpointFailure(String),
    InternalInvariantViolation(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    pub fn corruption(message: impl Into<String>) -> Self {
        Self::Corruption(message.into())
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest(message.into())
    }

    pub fn overloaded(message: impl Into<String>) -> Self {
        Self::Overloaded(message.into())
    }

    pub fn response_too_large(message: impl Into<String>) -> Self {
        Self::ResponseTooLarge(message.into())
    }

    pub fn conflict(conflict: crate::transaction::TransactionConflict) -> Self {
        Self::Conflict(conflict)
    }

    pub fn unsupported_format(message: impl Into<String>) -> Self {
        Self::UnsupportedFormat(message.into())
    }

    pub fn durability(message: impl Into<String>) -> Self {
        Self::DurabilityFailure(message.into())
    }

    pub fn recovery(message: impl Into<String>) -> Self {
        Self::RecoveryFailure(message.into())
    }

    pub fn checkpoint(message: impl Into<String>) -> Self {
        Self::CheckpointFailure(message.into())
    }

    pub fn invariant(message: impl Into<String>) -> Self {
        Self::InternalInvariantViolation(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(formatter, "invalid input: {message}"),
            Self::InvalidRequest(message) => write!(formatter, "invalid request: {message}"),
            Self::Overloaded(message) => write!(formatter, "overloaded: {message}"),
            Self::ResponseTooLarge(message) => write!(formatter, "response too large: {message}"),
            Self::Conflict(conflict) => write!(formatter, "conflict: {conflict}"),
            Self::Corruption(message) => write!(formatter, "corruption: {message}"),
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::UnsupportedFormat(message) => write!(formatter, "unsupported format: {message}"),
            Self::DurabilityFailure(message) => write!(formatter, "durability failure: {message}"),
            Self::RecoveryFailure(message) => write!(formatter, "recovery failure: {message}"),
            Self::CheckpointFailure(message) => {
                write!(formatter, "checkpoint failure: {message}")
            }
            Self::InternalInvariantViolation(message) => {
                write!(formatter, "internal invariant violation: {message}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
