use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemotePeeringConnectionPeeringStatus {
    #[serde(rename = "INVALID")]
    Invalid,

    #[serde(rename = "NEW")]
    New,

    #[serde(rename = "PENDING")]
    Pending,

    #[serde(rename = "PEERED")]
    Peered,

    #[serde(rename = "REVOKED")]
    Revoked,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
