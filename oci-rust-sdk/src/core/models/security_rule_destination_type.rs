use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityRuleDestinationType {
    #[serde(rename = "CIDR_BLOCK")]
    CidrBlock,

    #[serde(rename = "SERVICE_CIDR_BLOCK")]
    ServiceCidrBlock,

    #[serde(rename = "NETWORK_SECURITY_GROUP")]
    NetworkSecurityGroup,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
