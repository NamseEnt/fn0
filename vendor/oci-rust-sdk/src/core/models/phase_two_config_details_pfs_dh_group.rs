use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseTwoConfigDetailsPfsDhGroup {
    #[serde(rename = "GROUP2")]
    Group2,

    #[serde(rename = "GROUP5")]
    Group5,

    #[serde(rename = "GROUP14")]
    Group14,

    #[serde(rename = "GROUP19")]
    Group19,

    #[serde(rename = "GROUP20")]
    Group20,

    #[serde(rename = "GROUP24")]
    Group24,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
