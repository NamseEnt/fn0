use serde::{Deserialize, Serialize};

/// Address type of the CIDR/IP within a VCN or subnet
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressType {
    #[serde(rename = "Private_IPv4")]
    PrivateIPv4,

    #[serde(rename = "Oracle_Allocated_Public_IPv4")]
    OracleAllocatedPublicIPv4,

    #[serde(rename = "BYOIP_IPv4")]
    ByoipIPv4,

    #[serde(rename = "ULA_IPv6")]
    UlaIPv6,

    #[serde(rename = "Oracle_Allocated_GUA_IPv6")]
    OracleAllocatedGuaIPv6,

    #[serde(rename = "BYOIP_IPv6")]
    ByoipIPv6,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
