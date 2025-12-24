use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreateVtapDetailsTargetType {
    #[serde(rename = "VNIC")]
    Vnic,

    #[serde(rename = "NETWORK_LOAD_BALANCER")]
    NetworkLoadBalancer,

    #[serde(rename = "IP_ADDRESS")]
    IpAddress,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
