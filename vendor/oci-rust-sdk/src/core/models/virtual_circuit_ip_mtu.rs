use serde::{Deserialize, Serialize};

/// The layer 3 IP MTU to use with this virtual circuit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirtualCircuitIpMtu {
    #[serde(rename = "MTU_1500")]
    Mtu1500,

    #[serde(rename = "MTU_9000")]
    Mtu9000,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
