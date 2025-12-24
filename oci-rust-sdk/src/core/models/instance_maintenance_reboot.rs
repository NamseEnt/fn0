use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The maximum possible date and time that a maintenance reboot can be extended.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceMaintenanceReboot {
    /// The maximum extension date and time for the maintenance reboot, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). The range for the maintenance extension is between 1 and 14 days from the initial scheduled maintenance date. Example: {@code 2018-05-25T21:10:29.600Z}
    pub time_maintenance_reboot_due_max: DateTime<Utc>,
}

/// Required fields for InstanceMaintenanceReboot
pub struct InstanceMaintenanceRebootRequired {
    /// The maximum extension date and time for the maintenance reboot, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). The range for the maintenance extension is between 1 and 14 days from the initial scheduled maintenance date. Example: {@code 2018-05-25T21:10:29.600Z}
    pub time_maintenance_reboot_due_max: DateTime<Utc>,
}

impl InstanceMaintenanceReboot {
    /// Create a new InstanceMaintenanceReboot with required fields
    pub fn new(required: InstanceMaintenanceRebootRequired) -> Self {
        Self {
            time_maintenance_reboot_due_max: required.time_maintenance_reboot_due_max,
        }
    }

    /// Set time_maintenance_reboot_due_max
    pub fn set_time_maintenance_reboot_due_max(mut self, value: DateTime<Utc>) -> Self {
        self.time_maintenance_reboot_due_max = value;
        self
    }
}
