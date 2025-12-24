use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FastConnectProviderServiceType {
    #[serde(rename = "LAYER2")]
    Layer2,

    #[serde(rename = "LAYER3")]
    Layer3,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
