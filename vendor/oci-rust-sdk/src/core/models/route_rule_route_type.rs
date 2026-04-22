use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteRuleRouteType {
    #[serde(rename = "STATIC")]
    Static,

    #[serde(rename = "LOCAL")]
    Local,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
