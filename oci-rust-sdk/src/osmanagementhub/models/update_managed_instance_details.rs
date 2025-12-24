use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to update a managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateManagedInstanceDetails {
    /// User-specified description of the managed instance. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station for the instance to use as primary management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_management_station_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station for the instance to use as secondary management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_management_station_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Oracle Notifications service (ONS) topic. ONS is the channel used to send notifications to the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_topic_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub autonomous_settings: Option<UpdatableAutonomousSettings>,
}

impl UpdateManagedInstanceDetails {
    /// Create a new UpdateManagedInstanceDetails
    pub fn new() -> Self {
        Self {
            description: None,

            primary_management_station_id: None,

            secondary_management_station_id: None,

            notification_topic_id: None,

            autonomous_settings: None,
        }
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set primary_management_station_id
    pub fn set_primary_management_station_id(mut self, value: Option<String>) -> Self {
        self.primary_management_station_id = value;
        self
    }

    /// Set secondary_management_station_id
    pub fn set_secondary_management_station_id(mut self, value: Option<String>) -> Self {
        self.secondary_management_station_id = value;
        self
    }

    /// Set notification_topic_id
    pub fn set_notification_topic_id(mut self, value: Option<String>) -> Self {
        self.notification_topic_id = value;
        self
    }

    /// Set autonomous_settings
    pub fn set_autonomous_settings(mut self, value: Option<UpdatableAutonomousSettings>) -> Self {
        self.autonomous_settings = value;
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set primary_management_station_id (unwraps Option)
    pub fn with_primary_management_station_id(mut self, value: impl Into<String>) -> Self {
        self.primary_management_station_id = Some(value.into());
        self
    }

    /// Set secondary_management_station_id (unwraps Option)
    pub fn with_secondary_management_station_id(mut self, value: impl Into<String>) -> Self {
        self.secondary_management_station_id = Some(value.into());
        self
    }

    /// Set notification_topic_id (unwraps Option)
    pub fn with_notification_topic_id(mut self, value: impl Into<String>) -> Self {
        self.notification_topic_id = Some(value.into());
        self
    }

    /// Set autonomous_settings (unwraps Option)
    pub fn with_autonomous_settings(mut self, value: UpdatableAutonomousSettings) -> Self {
        self.autonomous_settings = Some(value);
        self
    }
}

impl Default for UpdateManagedInstanceDetails {
    fn default() -> Self {
        Self::new()
    }
}
