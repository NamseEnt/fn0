use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntelIcelakeBmPlatformConfigNumaNodesPerSocket {
    #[serde(rename = "NPS1")]
    Nps1,

    #[serde(rename = "NPS2")]
    Nps2,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
