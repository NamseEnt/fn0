use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeBillingType {
    #[serde(rename = "ALWAYS_FREE")]
    AlwaysFree,

    #[serde(rename = "LIMITED_FREE")]
    LimitedFree,

    #[serde(rename = "PAID")]
    Paid,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
