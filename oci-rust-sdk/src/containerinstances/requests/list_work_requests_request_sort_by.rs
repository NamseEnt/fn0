use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListWorkRequestsRequestSortBy {
    #[serde(rename = "timeAccepted")]
    TimeAccepted,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}

impl fmt::Display for ListWorkRequestsRequestSortBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TimeAccepted => write!(f, "timeAccepted"),

            Self::UnknownValue => write!(f, "UNKNOWN"),
        }
    }
}
