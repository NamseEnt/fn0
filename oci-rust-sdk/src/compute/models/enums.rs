use serde::{Deserialize, Serialize};

/// Lifecycle state of a compute instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LifecycleState {
    Moving,
    Provisioning,
    Running,
    Starting,
    Stopping,
    Stopped,
    CreatingImage,
    Terminating,
    Terminated,
}

/// Source type for instance boot volume
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceType {
    /// Boot from an image
    Image,
    /// Boot from a boot volume
    BootVolume,
}
