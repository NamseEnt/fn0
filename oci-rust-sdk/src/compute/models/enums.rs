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

impl std::fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifecycleState::Moving => write!(f, "MOVING"),
            LifecycleState::Provisioning => write!(f, "PROVISIONING"),
            LifecycleState::Running => write!(f, "RUNNING"),
            LifecycleState::Starting => write!(f, "STARTING"),
            LifecycleState::Stopping => write!(f, "STOPPING"),
            LifecycleState::Stopped => write!(f, "STOPPED"),
            LifecycleState::CreatingImage => write!(f, "CREATING_IMAGE"),
            LifecycleState::Terminating => write!(f, "TERMINATING"),
            LifecycleState::Terminated => write!(f, "TERMINATED"),
        }
    }
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

/// Sort field for listing instances
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    /// Sort by time created
    TimeCreated,
    /// Sort by display name
    DisplayName,
}

/// Sort order for listing instances
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}
