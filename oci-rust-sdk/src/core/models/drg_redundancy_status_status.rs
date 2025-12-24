use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrgRedundancyStatusStatus {
    #[serde(rename = "NOT_AVAILABLE")]
    NotAvailable,

    #[serde(rename = "REDUNDANT")]
    Redundant,

    #[serde(rename = "NOT_REDUNDANT_SINGLE_IPSEC")]
    NotRedundantSingleIpsec,

    #[serde(rename = "NOT_REDUNDANT_SINGLE_VIRTUALCIRCUIT")]
    NotRedundantSingleVirtualcircuit,

    #[serde(rename = "NOT_REDUNDANT_MULTIPLE_IPSECS")]
    NotRedundantMultipleIpsecs,

    #[serde(rename = "NOT_REDUNDANT_MULTIPLE_VIRTUALCIRCUITS")]
    NotRedundantMultipleVirtualcircuits,

    #[serde(rename = "NOT_REDUNDANT_MIX_CONNECTIONS")]
    NotRedundantMixConnections,

    #[serde(rename = "NOT_REDUNDANT_NO_CONNECTION")]
    NotRedundantNoConnection,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
