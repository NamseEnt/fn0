use serde::{Deserialize, Serialize};

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
