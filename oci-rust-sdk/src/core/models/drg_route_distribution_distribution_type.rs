use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrgRouteDistributionDistributionType {
    #[serde(rename = "IMPORT")]
    Import,

    #[serde(rename = "EXPORT")]
    Export,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
