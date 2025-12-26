use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceConfigurationAmdMilanBmLaunchInstancePlatformConfigNumaNodesPerSocket {
    #[serde(rename = "NPS0")]
    Nps0,

    #[serde(rename = "NPS1")]
    Nps1,

    #[serde(rename = "NPS2")]
    Nps2,

    #[serde(rename = "NPS4")]
    Nps4,

    #[serde(rename = "NPS6")]
    Nps6,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
