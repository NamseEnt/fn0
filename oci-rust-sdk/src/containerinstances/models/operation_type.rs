use serde::{Deserialize, Serialize};

/// Possible operation types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    #[serde(rename = "CREATE_CONTAINER_INSTANCE")]
    CreateContainerInstance,

    #[serde(rename = "UPDATE_CONTAINER_INSTANCE")]
    UpdateContainerInstance,

    #[serde(rename = "DELETE_CONTAINER_INSTANCE")]
    DeleteContainerInstance,

    #[serde(rename = "MOVE_CONTAINER_INSTANCE")]
    MoveContainerInstance,

    #[serde(rename = "START_CONTAINER_INSTANCE")]
    StartContainerInstance,

    #[serde(rename = "STOP_CONTAINER_INSTANCE")]
    StopContainerInstance,

    #[serde(rename = "RESTART_CONTAINER_INSTANCE")]
    RestartContainerInstance,

    #[serde(rename = "UPDATE_CONTAINER")]
    UpdateContainer,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
