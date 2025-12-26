use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrgRouteRuleRouteProvenance {
    #[serde(rename = "STATIC")]
    Static,

    #[serde(rename = "VCN")]
    Vcn,

    #[serde(rename = "VIRTUAL_CIRCUIT")]
    VirtualCircuit,

    #[serde(rename = "IPSEC_TUNNEL")]
    IpsecTunnel,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
