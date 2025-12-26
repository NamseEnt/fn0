use serde::{Deserialize, Serialize};
use std::fmt;

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

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateContainerInstance => write!(f, "CREATE_CONTAINER_INSTANCE"),

            Self::UpdateContainerInstance => write!(f, "UPDATE_CONTAINER_INSTANCE"),

            Self::DeleteContainerInstance => write!(f, "DELETE_CONTAINER_INSTANCE"),

            Self::MoveContainerInstance => write!(f, "MOVE_CONTAINER_INSTANCE"),

            Self::StartContainerInstance => write!(f, "START_CONTAINER_INSTANCE"),

            Self::StopContainerInstance => write!(f, "STOP_CONTAINER_INSTANCE"),

            Self::RestartContainerInstance => write!(f, "RESTART_CONTAINER_INSTANCE"),

            Self::UpdateContainer => write!(f, "UPDATE_CONTAINER"),

            Self::UnknownValue => write!(f, "UNKNOWN"),
        }
    }
}
