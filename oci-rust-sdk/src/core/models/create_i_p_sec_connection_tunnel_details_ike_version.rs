use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreateIPSecConnectionTunnelDetailsIkeVersion {
    #[serde(rename = "V1")]
    V1,

    #[serde(rename = "V2")]
    V2,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
