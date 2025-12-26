use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstancePoolInstanceLoadBalancerBackendBackendHealthStatus {
    #[serde(rename = "OK")]
    Ok,

    #[serde(rename = "WARNING")]
    Warning,

    #[serde(rename = "CRITICAL")]
    Critical,

    #[serde(rename = "UNKNOWN")]
    Unknown,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
