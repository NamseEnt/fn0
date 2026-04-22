use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information for a managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the managed instance.
    pub id: String,

    /// User-friendly name for the managed instance.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the tenancy this managed instance resides in.
    pub tenancy_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the managed instance.
    pub compartment_id: String,

    /// Current status of the managed instance.
    pub status: ManagedInstanceStatus,

    /// User-specified description of the managed instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The location of the managed instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<ManagedInstanceLocation>,

    /// The CPU architecture type of the managed instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<ArchType>,

    /// The operating system type of the managed instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_family: Option<OsFamily>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_instance_group: Option<Id>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_environment: Option<Id>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_stage: Option<Id>,

    /// Indicates whether a reboot is required to complete installation of updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_reboot_required: Option<bool>,

    /// Number of updates available for installation. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updates_available: Option<i64>,

    /// Whether this managed instance is acting as an on-premises management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_management_station: Option<bool>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Oracle Notifications service (ONS) topic. ONS is the channel used to send notifications to the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_topic_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub autonomous_settings: Option<AutonomousSettings>,

    /// Indicates whether Autonomous Linux manages this instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_managed_by_autonomous_linux: Option<bool>,

    /// The version of osmh-agent running on the managed instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_version: Option<String>,
}

/// Required fields for ManagedInstanceSummary
pub struct ManagedInstanceSummaryRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the managed instance.
    pub id: String,

    /// User-friendly name for the managed instance.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the tenancy this managed instance resides in.
    pub tenancy_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the managed instance.
    pub compartment_id: String,

    /// Current status of the managed instance.
    pub status: ManagedInstanceStatus,
}

impl ManagedInstanceSummary {
    /// Create a new ManagedInstanceSummary with required fields
    pub fn new(required: ManagedInstanceSummaryRequired) -> Self {
        Self {
            id: required.id,

            display_name: required.display_name,

            tenancy_id: required.tenancy_id,

            compartment_id: required.compartment_id,

            status: required.status,

            description: None,

            location: None,

            architecture: None,

            os_family: None,

            managed_instance_group: None,

            lifecycle_environment: None,

            lifecycle_stage: None,

            is_reboot_required: None,

            updates_available: None,

            is_management_station: None,

            notification_topic_id: None,

            autonomous_settings: None,

            is_managed_by_autonomous_linux: None,

            agent_version: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set tenancy_id
    pub fn set_tenancy_id(mut self, value: String) -> Self {
        self.tenancy_id = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set location
    pub fn set_location(mut self, value: Option<ManagedInstanceLocation>) -> Self {
        self.location = value;
        self
    }

    /// Set architecture
    pub fn set_architecture(mut self, value: Option<ArchType>) -> Self {
        self.architecture = value;
        self
    }

    /// Set os_family
    pub fn set_os_family(mut self, value: Option<OsFamily>) -> Self {
        self.os_family = value;
        self
    }

    /// Set status
    pub fn set_status(mut self, value: ManagedInstanceStatus) -> Self {
        self.status = value;
        self
    }

    /// Set managed_instance_group
    pub fn set_managed_instance_group(mut self, value: Option<Id>) -> Self {
        self.managed_instance_group = value;
        self
    }

    /// Set lifecycle_environment
    pub fn set_lifecycle_environment(mut self, value: Option<Id>) -> Self {
        self.lifecycle_environment = value;
        self
    }

    /// Set lifecycle_stage
    pub fn set_lifecycle_stage(mut self, value: Option<Id>) -> Self {
        self.lifecycle_stage = value;
        self
    }

    /// Set is_reboot_required
    pub fn set_is_reboot_required(mut self, value: Option<bool>) -> Self {
        self.is_reboot_required = value;
        self
    }

    /// Set updates_available
    pub fn set_updates_available(mut self, value: Option<i64>) -> Self {
        self.updates_available = value;
        self
    }

    /// Set is_management_station
    pub fn set_is_management_station(mut self, value: Option<bool>) -> Self {
        self.is_management_station = value;
        self
    }

    /// Set notification_topic_id
    pub fn set_notification_topic_id(mut self, value: Option<String>) -> Self {
        self.notification_topic_id = value;
        self
    }

    /// Set autonomous_settings
    pub fn set_autonomous_settings(mut self, value: Option<AutonomousSettings>) -> Self {
        self.autonomous_settings = value;
        self
    }

    /// Set is_managed_by_autonomous_linux
    pub fn set_is_managed_by_autonomous_linux(mut self, value: Option<bool>) -> Self {
        self.is_managed_by_autonomous_linux = value;
        self
    }

    /// Set agent_version
    pub fn set_agent_version(mut self, value: Option<String>) -> Self {
        self.agent_version = value;
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set location (unwraps Option)
    pub fn with_location(mut self, value: ManagedInstanceLocation) -> Self {
        self.location = Some(value);
        self
    }

    /// Set architecture (unwraps Option)
    pub fn with_architecture(mut self, value: ArchType) -> Self {
        self.architecture = Some(value);
        self
    }

    /// Set os_family (unwraps Option)
    pub fn with_os_family(mut self, value: OsFamily) -> Self {
        self.os_family = Some(value);
        self
    }

    /// Set managed_instance_group (unwraps Option)
    pub fn with_managed_instance_group(mut self, value: Id) -> Self {
        self.managed_instance_group = Some(value);
        self
    }

    /// Set lifecycle_environment (unwraps Option)
    pub fn with_lifecycle_environment(mut self, value: Id) -> Self {
        self.lifecycle_environment = Some(value);
        self
    }

    /// Set lifecycle_stage (unwraps Option)
    pub fn with_lifecycle_stage(mut self, value: Id) -> Self {
        self.lifecycle_stage = Some(value);
        self
    }

    /// Set is_reboot_required (unwraps Option)
    pub fn with_is_reboot_required(mut self, value: bool) -> Self {
        self.is_reboot_required = Some(value);
        self
    }

    /// Set updates_available (unwraps Option)
    pub fn with_updates_available(mut self, value: i64) -> Self {
        self.updates_available = Some(value);
        self
    }

    /// Set is_management_station (unwraps Option)
    pub fn with_is_management_station(mut self, value: bool) -> Self {
        self.is_management_station = Some(value);
        self
    }

    /// Set notification_topic_id (unwraps Option)
    pub fn with_notification_topic_id(mut self, value: impl Into<String>) -> Self {
        self.notification_topic_id = Some(value.into());
        self
    }

    /// Set autonomous_settings (unwraps Option)
    pub fn with_autonomous_settings(mut self, value: AutonomousSettings) -> Self {
        self.autonomous_settings = Some(value);
        self
    }

    /// Set is_managed_by_autonomous_linux (unwraps Option)
    pub fn with_is_managed_by_autonomous_linux(mut self, value: bool) -> Self {
        self.is_managed_by_autonomous_linux = Some(value);
        self
    }

    /// Set agent_version (unwraps Option)
    pub fn with_agent_version(mut self, value: impl Into<String>) -> Self {
        self.agent_version = Some(value.into());
        self
    }
}
