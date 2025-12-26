use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateInstanceShapeConfigDetailsBaselineOcpuUtilization {
    #[serde(rename = "BASELINE_1_8")]
    Baseline18,

    #[serde(rename = "BASELINE_1_2")]
    Baseline12,

    #[serde(rename = "BASELINE_1_1")]
    Baseline11,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
