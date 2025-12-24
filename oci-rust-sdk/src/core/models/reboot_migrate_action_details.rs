use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Parameters for the {@code rebootMigrate} {@link #instanceAction(InstanceActionRequest) instanceAction}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebootMigrateActionDetails {
    pub action_type: String,

    /// For bare metal instances that have local storage, this must be set to true to verify that the local storage will be deleted during the migration.  For instances without, this parameter has no effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_local_storage: Option<bool>,

    /// If present, this parameter will set (or reset) the scheduled time that the instance will be reboot migrated in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).  This will also change the {@code timeMaintenanceRebootDue} field on the instance. <p> If not present, the reboot migration will be triggered immediately.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_scheduled: Option<DateTime<Utc>>,
}

/// Required fields for RebootMigrateActionDetails
pub struct RebootMigrateActionDetailsRequired {
    pub action_type: String,
}

impl RebootMigrateActionDetails {
    /// Create a new RebootMigrateActionDetails with required fields
    pub fn new(required: RebootMigrateActionDetailsRequired) -> Self {
        Self {
            action_type: required.action_type,

            delete_local_storage: None,

            time_scheduled: None,
        }
    }

    /// Set delete_local_storage
    pub fn set_delete_local_storage(mut self, value: Option<bool>) -> Self {
        self.delete_local_storage = value;
        self
    }

    /// Set time_scheduled
    pub fn set_time_scheduled(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_scheduled = value;
        self
    }

    /// Set action_type
    pub fn set_action_type(mut self, value: String) -> Self {
        self.action_type = value;
        self
    }

    /// Set delete_local_storage (unwraps Option)
    pub fn with_delete_local_storage(mut self, value: bool) -> Self {
        self.delete_local_storage = Some(value);
        self
    }

    /// Set time_scheduled (unwraps Option)
    pub fn with_time_scheduled(mut self, value: DateTime<Utc>) -> Self {
        self.time_scheduled = Some(value);
        self
    }
}
