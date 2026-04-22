use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Parameters for the {@code reset} {@link #instanceAction(InstanceActionRequest) instanceAction}. If omitted, default values are used.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetActionDetails {
    pub action_type: String,

    /// For instances that use a DenseIO shape, the flag denoting whether [reboot migration](https://docs.oracle.com/iaas/Content/Compute/References/infrastructure-maintenance.htm#reboot) is performed for the instance. The default value is {@code false}. <p> If the instance has a date in the Maintenance reboot field and you do nothing (or set this flag to {@code false}), the instance will be rebuilt at the scheduled maintenance time. The instance will experience 2-6 hours of downtime during the maintenance process. The local NVMe-based SSD will be preserved. <p> If you want to minimize downtime and can delete the SSD, you can set this flag to {@code true} and proactively reboot the instance before the scheduled maintenance time. The instance will be reboot migrated to a healthy host and the SSD will be deleted. A short downtime occurs during the migration. <p> *Caution:** When {@code true}, the SSD is permanently deleted. We recommend that you create a backup of the SSD before proceeding.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_dense_reboot_migration: Option<bool>,
}

/// Required fields for ResetActionDetails
pub struct ResetActionDetailsRequired {
    pub action_type: String,
}

impl ResetActionDetails {
    /// Create a new ResetActionDetails with required fields
    pub fn new(required: ResetActionDetailsRequired) -> Self {
        Self {
            action_type: required.action_type,

            allow_dense_reboot_migration: None,
        }
    }

    /// Set allow_dense_reboot_migration
    pub fn set_allow_dense_reboot_migration(mut self, value: Option<bool>) -> Self {
        self.allow_dense_reboot_migration = value;
        self
    }

    /// Set action_type
    pub fn set_action_type(mut self, value: String) -> Self {
        self.action_type = value;
        self
    }

    /// Set allow_dense_reboot_migration (unwraps Option)
    pub fn with_allow_dense_reboot_migration(mut self, value: bool) -> Self {
        self.allow_dense_reboot_migration = Some(value);
        self
    }
}
