use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerInstanceLifecycleState {
    #[serde(rename = "CREATING")]
    Creating,

    #[serde(rename = "UPDATING")]
    Updating,

    #[serde(rename = "ACTIVE")]
    Active,

    #[serde(rename = "INACTIVE")]
    Inactive,

    #[serde(rename = "DELETING")]
    Deleting,

    #[serde(rename = "DELETED")]
    Deleted,

    #[serde(rename = "FAILED")]
    Failed,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}

impl fmt::Display for ContainerInstanceLifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Creating => write!(f, "CREATING"),

            Self::Updating => write!(f, "UPDATING"),

            Self::Active => write!(f, "ACTIVE"),

            Self::Inactive => write!(f, "INACTIVE"),

            Self::Deleting => write!(f, "DELETING"),

            Self::Deleted => write!(f, "DELETED"),

            Self::Failed => write!(f, "FAILED"),

            Self::UnknownValue => write!(f, "UNKNOWN"),
        }
    }
}
