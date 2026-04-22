use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An object that defines the instance being managed by the service.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstance {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the managed instance.
    pub id: String,

    /// User-friendly name for the managed instance.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the tenancy that the managed instance resides in.
    pub tenancy_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the managed instance.
    pub compartment_id: String,

    /// Current status of the managed instance.
    pub status: ManagedInstanceStatus,

    /// User-specified description for the managed instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The location of the managed instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<ManagedInstanceLocation>,

    /// Time that the instance last checked in with the service (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_last_checkin: Option<DateTime<Utc>>,

    /// Time that the instance last booted (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_last_boot: Option<DateTime<Utc>>,

    /// Operating system name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_name: Option<String>,

    /// Operating system version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,

    /// Operating system kernel version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_kernel_version: Option<String>,

    /// The ksplice effective kernel version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ksplice_effective_kernel_version: Option<String>,

    /// The CPU architecture type of the managed instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<ArchType>,

    /// The operating system type of the managed instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_family: Option<OsFamily>,

    /// The profile that was used to register this instance with the service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,

    /// The version of the profile that was used to register this instance with the service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_version: Option<String>,

    /// Indicates whether this managed instance is acting as an on-premises management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_management_station: Option<bool>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station for the instance to use as primary management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_management_station_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station for the instance to use as secondary management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_management_station_id: Option<String>,

    /// The list of software sources currently attached to the managed instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_sources: Option<Vec<SoftwareSourceDetails>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_instance_group: Option<Id>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_environment: Option<Id>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_stage: Option<Id>,

    /// Indicates whether a reboot is required to complete installation of updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_reboot_required: Option<bool>,

    /// Number of packages installed on the instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_packages: Option<i64>,

    /// Number of Windows updates installed on the instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_windows_updates: Option<i64>,

    /// Number of updates available for installation. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updates_available: Option<i64>,

    /// Number of security type updates available for installation. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_updates_available: Option<i64>,

    /// Number of bug fix type updates available for installation. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bug_updates_available: Option<i64>,

    /// Number of enhancement type updates available for installation. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhancement_updates_available: Option<i64>,

    /// Number of non-classified (other) updates available for installation. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other_updates_available: Option<i64>,

    /// Number of scheduled jobs associated with this instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_job_count: Option<i64>,

    /// Number of work requests associated with this instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_count: Option<i64>,

    /// The date and time the instance was created (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// The date and time the instance was last updated (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_updated: Option<DateTime<Utc>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Oracle Notifications service (ONS) topic. ONS is the channel used to send notifications to the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_topic_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub autonomous_settings: Option<AutonomousSettings>,

    /// Indicates whether the Autonomous Linux service manages the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_managed_by_autonomous_linux: Option<bool>,

    /// The version of osmh-agent running on the managed instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_version: Option<String>,
}

/// Required fields for ManagedInstance
pub struct ManagedInstanceRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the managed instance.
    pub id: String,

    /// User-friendly name for the managed instance.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the tenancy that the managed instance resides in.
    pub tenancy_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the managed instance.
    pub compartment_id: String,

    /// Current status of the managed instance.
    pub status: ManagedInstanceStatus,
}

impl ManagedInstance {
    /// Create a new ManagedInstance with required fields
    pub fn new(required: ManagedInstanceRequired) -> Self {
        Self {
            id: required.id,

            display_name: required.display_name,

            tenancy_id: required.tenancy_id,

            compartment_id: required.compartment_id,

            status: required.status,

            description: None,

            location: None,

            time_last_checkin: None,

            time_last_boot: None,

            os_name: None,

            os_version: None,

            os_kernel_version: None,

            ksplice_effective_kernel_version: None,

            architecture: None,

            os_family: None,

            profile: None,

            profile_version: None,

            is_management_station: None,

            primary_management_station_id: None,

            secondary_management_station_id: None,

            software_sources: None,

            managed_instance_group: None,

            lifecycle_environment: None,

            lifecycle_stage: None,

            is_reboot_required: None,

            installed_packages: None,

            installed_windows_updates: None,

            updates_available: None,

            security_updates_available: None,

            bug_updates_available: None,

            enhancement_updates_available: None,

            other_updates_available: None,

            scheduled_job_count: None,

            work_request_count: None,

            time_created: None,

            time_updated: None,

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

    /// Set time_last_checkin
    pub fn set_time_last_checkin(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_last_checkin = value;
        self
    }

    /// Set time_last_boot
    pub fn set_time_last_boot(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_last_boot = value;
        self
    }

    /// Set os_name
    pub fn set_os_name(mut self, value: Option<String>) -> Self {
        self.os_name = value;
        self
    }

    /// Set os_version
    pub fn set_os_version(mut self, value: Option<String>) -> Self {
        self.os_version = value;
        self
    }

    /// Set os_kernel_version
    pub fn set_os_kernel_version(mut self, value: Option<String>) -> Self {
        self.os_kernel_version = value;
        self
    }

    /// Set ksplice_effective_kernel_version
    pub fn set_ksplice_effective_kernel_version(mut self, value: Option<String>) -> Self {
        self.ksplice_effective_kernel_version = value;
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

    /// Set profile
    pub fn set_profile(mut self, value: Option<String>) -> Self {
        self.profile = value;
        self
    }

    /// Set profile_version
    pub fn set_profile_version(mut self, value: Option<String>) -> Self {
        self.profile_version = value;
        self
    }

    /// Set is_management_station
    pub fn set_is_management_station(mut self, value: Option<bool>) -> Self {
        self.is_management_station = value;
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

    /// Set software_sources
    pub fn set_software_sources(mut self, value: Option<Vec<SoftwareSourceDetails>>) -> Self {
        self.software_sources = value;
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

    /// Set installed_packages
    pub fn set_installed_packages(mut self, value: Option<i64>) -> Self {
        self.installed_packages = value;
        self
    }

    /// Set installed_windows_updates
    pub fn set_installed_windows_updates(mut self, value: Option<i64>) -> Self {
        self.installed_windows_updates = value;
        self
    }

    /// Set updates_available
    pub fn set_updates_available(mut self, value: Option<i64>) -> Self {
        self.updates_available = value;
        self
    }

    /// Set security_updates_available
    pub fn set_security_updates_available(mut self, value: Option<i64>) -> Self {
        self.security_updates_available = value;
        self
    }

    /// Set bug_updates_available
    pub fn set_bug_updates_available(mut self, value: Option<i64>) -> Self {
        self.bug_updates_available = value;
        self
    }

    /// Set enhancement_updates_available
    pub fn set_enhancement_updates_available(mut self, value: Option<i64>) -> Self {
        self.enhancement_updates_available = value;
        self
    }

    /// Set other_updates_available
    pub fn set_other_updates_available(mut self, value: Option<i64>) -> Self {
        self.other_updates_available = value;
        self
    }

    /// Set scheduled_job_count
    pub fn set_scheduled_job_count(mut self, value: Option<i64>) -> Self {
        self.scheduled_job_count = value;
        self
    }

    /// Set work_request_count
    pub fn set_work_request_count(mut self, value: Option<i64>) -> Self {
        self.work_request_count = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set time_updated
    pub fn set_time_updated(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_updated = value;
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

    /// Set time_last_checkin (unwraps Option)
    pub fn with_time_last_checkin(mut self, value: DateTime<Utc>) -> Self {
        self.time_last_checkin = Some(value);
        self
    }

    /// Set time_last_boot (unwraps Option)
    pub fn with_time_last_boot(mut self, value: DateTime<Utc>) -> Self {
        self.time_last_boot = Some(value);
        self
    }

    /// Set os_name (unwraps Option)
    pub fn with_os_name(mut self, value: impl Into<String>) -> Self {
        self.os_name = Some(value.into());
        self
    }

    /// Set os_version (unwraps Option)
    pub fn with_os_version(mut self, value: impl Into<String>) -> Self {
        self.os_version = Some(value.into());
        self
    }

    /// Set os_kernel_version (unwraps Option)
    pub fn with_os_kernel_version(mut self, value: impl Into<String>) -> Self {
        self.os_kernel_version = Some(value.into());
        self
    }

    /// Set ksplice_effective_kernel_version (unwraps Option)
    pub fn with_ksplice_effective_kernel_version(mut self, value: impl Into<String>) -> Self {
        self.ksplice_effective_kernel_version = Some(value.into());
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

    /// Set profile (unwraps Option)
    pub fn with_profile(mut self, value: impl Into<String>) -> Self {
        self.profile = Some(value.into());
        self
    }

    /// Set profile_version (unwraps Option)
    pub fn with_profile_version(mut self, value: impl Into<String>) -> Self {
        self.profile_version = Some(value.into());
        self
    }

    /// Set is_management_station (unwraps Option)
    pub fn with_is_management_station(mut self, value: bool) -> Self {
        self.is_management_station = Some(value);
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

    /// Set software_sources (unwraps Option)
    pub fn with_software_sources(mut self, value: Vec<SoftwareSourceDetails>) -> Self {
        self.software_sources = Some(value);
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

    /// Set installed_packages (unwraps Option)
    pub fn with_installed_packages(mut self, value: i64) -> Self {
        self.installed_packages = Some(value);
        self
    }

    /// Set installed_windows_updates (unwraps Option)
    pub fn with_installed_windows_updates(mut self, value: i64) -> Self {
        self.installed_windows_updates = Some(value);
        self
    }

    /// Set updates_available (unwraps Option)
    pub fn with_updates_available(mut self, value: i64) -> Self {
        self.updates_available = Some(value);
        self
    }

    /// Set security_updates_available (unwraps Option)
    pub fn with_security_updates_available(mut self, value: i64) -> Self {
        self.security_updates_available = Some(value);
        self
    }

    /// Set bug_updates_available (unwraps Option)
    pub fn with_bug_updates_available(mut self, value: i64) -> Self {
        self.bug_updates_available = Some(value);
        self
    }

    /// Set enhancement_updates_available (unwraps Option)
    pub fn with_enhancement_updates_available(mut self, value: i64) -> Self {
        self.enhancement_updates_available = Some(value);
        self
    }

    /// Set other_updates_available (unwraps Option)
    pub fn with_other_updates_available(mut self, value: i64) -> Self {
        self.other_updates_available = Some(value);
        self
    }

    /// Set scheduled_job_count (unwraps Option)
    pub fn with_scheduled_job_count(mut self, value: i64) -> Self {
        self.scheduled_job_count = Some(value);
        self
    }

    /// Set work_request_count (unwraps Option)
    pub fn with_work_request_count(mut self, value: i64) -> Self {
        self.work_request_count = Some(value);
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }

    /// Set time_updated (unwraps Option)
    pub fn with_time_updated(mut self, value: DateTime<Utc>) -> Self {
        self.time_updated = Some(value);
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
