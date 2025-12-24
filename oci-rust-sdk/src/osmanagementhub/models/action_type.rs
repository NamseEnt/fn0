use serde::{Deserialize, Serialize};

/// Possible types of actions on the target resource.
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
