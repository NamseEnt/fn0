use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IPSecConnectionCpeLocalIdentifierType {
    #[serde(rename = "IP_ADDRESS")]
    IpAddress,

    #[serde(rename = "HOSTNAME")]
    Hostname,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
