use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirtualCircuitRedundancyMetadataIpv6bgpSessionRedundancyStatus {
    #[serde(rename = "CONFIGURATION_MATCH")]
    ConfigurationMatch,

    #[serde(rename = "CONFIGURATION_MISMATCH")]
    ConfigurationMismatch,

    #[serde(rename = "NOT_MET_SLA")]
    NotMetSla,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
