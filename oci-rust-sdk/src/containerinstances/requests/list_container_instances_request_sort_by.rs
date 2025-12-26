use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListContainerInstancesRequestSortBy {
    #[serde(rename = "timeCreated")]
    TimeCreated,

    #[serde(rename = "displayName")]
    DisplayName,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}

impl fmt::Display for ListContainerInstancesRequestSortBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TimeCreated => write!(f, "timeCreated"),

            Self::DisplayName => write!(f, "displayName"),

            Self::UnknownValue => write!(f, "UNKNOWN"),
        }
    }
}
