use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicIpAssignedEntityType {
    #[serde(rename = "PRIVATE_IP")]
    PrivateIp,

    #[serde(rename = "NAT_GATEWAY")]
    NatGateway,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
