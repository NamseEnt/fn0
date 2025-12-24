use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Options for defining the availabiity of a VM instance after a maintenance event that impacts the underlying hardware.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationAvailabilityConfig {
    /// Whether to live migrate supported VM instances to a healthy physical VM host without disrupting running instances during infrastructure maintenance events. If null, Oracle chooses the best option for migrating the VM during infrastructure maintenance events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_live_migration_preferred: Option<bool>,

    /// The lifecycle state for an instance when it is recovered after infrastructure maintenance. * {@code RESTORE_INSTANCE} - The instance is restored to the lifecycle state it was in before the maintenance event. If the instance was running, it is automatically rebooted. This is the default action when a value is not set. * {@code STOP_INSTANCE} - The instance is recovered in the stopped state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_action: Option<InstanceConfigurationAvailabilityConfigRecoveryAction>,
}

impl InstanceConfigurationAvailabilityConfig {
    /// Create a new InstanceConfigurationAvailabilityConfig
    pub fn new() -> Self {
        Self {
            is_live_migration_preferred: None,

            recovery_action: None,
        }
    }

    /// Set is_live_migration_preferred
    pub fn set_is_live_migration_preferred(mut self, value: Option<bool>) -> Self {
        self.is_live_migration_preferred = value;
        self
    }

    /// Set recovery_action
    pub fn set_recovery_action(
        mut self,
        value: Option<InstanceConfigurationAvailabilityConfigRecoveryAction>,
    ) -> Self {
        self.recovery_action = value;
        self
    }

    /// Set is_live_migration_preferred (unwraps Option)
    pub fn with_is_live_migration_preferred(mut self, value: bool) -> Self {
        self.is_live_migration_preferred = Some(value);
        self
    }

    /// Set recovery_action (unwraps Option)
    pub fn with_recovery_action(
        mut self,
        value: InstanceConfigurationAvailabilityConfigRecoveryAction,
    ) -> Self {
        self.recovery_action = Some(value);
        self
    }
}

impl Default for InstanceConfigurationAvailabilityConfig {
    fn default() -> Self {
        Self::new()
    }
}
