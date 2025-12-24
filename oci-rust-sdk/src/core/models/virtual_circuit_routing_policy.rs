use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirtualCircuitRoutingPolicy {
    #[serde(rename = "ORACLE_SERVICE_NETWORK")]
    OracleServiceNetwork,

    #[serde(rename = "REGIONAL")]
    Regional,

    #[serde(rename = "MARKET_LEVEL")]
    MarketLevel,

    #[serde(rename = "GLOBAL")]
    Global,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
