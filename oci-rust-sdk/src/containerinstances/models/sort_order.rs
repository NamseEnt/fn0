use serde::{Deserialize, Serialize};
use std::fmt;

/// Sort orders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    #[serde(rename = "ASC")]
    Asc,

    #[serde(rename = "DESC")]
    Desc,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortOrder::Asc => write!(f, "ASC"),
            SortOrder::Desc => write!(f, "DESC"),
            SortOrder::UnknownValue => write!(f, "UNKNOWN"),
        }
    }
}
