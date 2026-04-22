use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpInventorySubnetResourceSummaryAssociatedPublicIpPool {
    #[serde(rename = "ORACLE")]
    Oracle,

    #[serde(rename = "BYOIP")]
    Byoip,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
