use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsoleHistoryLifecycleState {
    #[serde(rename = "REQUESTED")]
    Requested,

    #[serde(rename = "GETTING-HISTORY")]
    GettingHistory,

    #[serde(rename = "SUCCEEDED")]
    Succeeded,

    #[serde(rename = "FAILED")]
    Failed,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
