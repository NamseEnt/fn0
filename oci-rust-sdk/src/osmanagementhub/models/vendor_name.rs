use serde::{Deserialize, Serialize};

/// Name of OS vendor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VendorName {
    #[serde(rename = "ORACLE")]
    Oracle,

    #[serde(rename = "MICROSOFT")]
    Microsoft,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
