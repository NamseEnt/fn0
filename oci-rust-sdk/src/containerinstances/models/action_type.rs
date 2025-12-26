use serde::{Deserialize, Serialize};
use std::fmt;

/// Possible types of actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    #[serde(rename = "CREATED")]
    Created,

    #[serde(rename = "UPDATED")]
    Updated,

    #[serde(rename = "DELETED")]
    Deleted,

    #[serde(rename = "IN_PROGRESS")]
    InProgress,

    #[serde(rename = "RELATED")]
    Related,

    #[serde(rename = "FAILED")]
    Failed,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "CREATED"),

            Self::Updated => write!(f, "UPDATED"),

            Self::Deleted => write!(f, "DELETED"),

            Self::InProgress => write!(f, "IN_PROGRESS"),

            Self::Related => write!(f, "RELATED"),

            Self::Failed => write!(f, "FAILED"),

            Self::UnknownValue => write!(f, "UNKNOWN"),
        }
    }
}
