use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListIpInventoryDetailsSortBy {
    #[serde(rename = "DISPLAYNAME")]
    Displayname,

    #[serde(rename = "UTILIZATION")]
    Utilization,

    #[serde(rename = "DNS_HOSTNAME")]
    DnsHostname,

    #[serde(rename = "REGION")]
    Region,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
