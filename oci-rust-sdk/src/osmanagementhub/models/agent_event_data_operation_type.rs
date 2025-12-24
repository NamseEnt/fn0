use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentEventDataOperationType {
    #[serde(rename = "LIST_PACKAGES")]
    ListPackages,

    #[serde(rename = "UPLOAD_CONTENT")]
    UploadContent,

    #[serde(rename = "SYNC_AGENT_CONFIG")]
    SyncAgentConfig,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
