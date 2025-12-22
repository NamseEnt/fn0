use serde::{Deserialize, Serialize};

/// Options for defining the availability of a VM instance after a maintenance event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationAvailabilityConfig {
    /// Whether to live migrate supported VM instances to a healthy physical VM host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_live_migration_preferred: Option<bool>,

    /// The lifecycle state for an instance when it is recovered after infrastructure maintenance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_action: Option<RecoveryAction>,
}

/// The recovery action for an instance after infrastructure maintenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecoveryAction {
    /// The instance is restored to the lifecycle state it was in before the maintenance event.
    RestoreInstance,
    /// The instance is recovered in the stopped state.
    StopInstance,
}

impl InstanceConfigurationAvailabilityConfig {
    pub fn new() -> Self {
        Self {
            is_live_migration_preferred: None,
            recovery_action: None,
        }
    }

    pub fn with_is_live_migration_preferred(mut self, preferred: bool) -> Self {
        self.is_live_migration_preferred = Some(preferred);
        self
    }

    pub fn with_recovery_action(mut self, action: RecoveryAction) -> Self {
        self.recovery_action = Some(action);
        self
    }

    /// Set is_live_migration_preferred
    pub fn set_is_live_migration_preferred(mut self, is_live_migration_preferred: Option<bool>) -> Self {
        self.is_live_migration_preferred = is_live_migration_preferred;
        self
    }

    /// Set recovery_action
    pub fn set_recovery_action(mut self, recovery_action: Option<RecoveryAction>) -> Self {
        self.recovery_action = recovery_action;
        self
    }
}

impl Default for InstanceConfigurationAvailabilityConfig {
    fn default() -> Self {
        Self::new()
    }
}
