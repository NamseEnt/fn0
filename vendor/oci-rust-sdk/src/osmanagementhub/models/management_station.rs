use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Provides information about the management station, including name, state, and configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementStation {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the management station.
    pub compartment_id: String,

    /// A user-friendly name for the management station.
    pub display_name: String,

    /// Hostname of the management station.
    pub hostname: String,

    pub proxy: ProxyConfiguration,

    pub mirror: MirrorConfiguration,

    /// A list of other management stations that are behind the same load balancer within a high availability configuration. Stations are identified as peers if they have the same hostname and compartment.
    pub peer_management_stations: Vec<PeerManagementStation>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance that is acting as the management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_instance_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the scheduled job for the mirror sync.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_job_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the registration profile used for the management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,

    /// User-specified description for the management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Current state of the mirror sync for the management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overall_state: Option<OverallState>,

    /// A decimal number representing the progress of the current mirror sync. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overall_percentage: Option<i64>,

    /// A decimal number representing the amount of mirror capacity used by the sync. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror_capacity: Option<i64>,

    /// The number of software sources that the station is mirroring. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_mirrors: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror_sync_status: Option<MirrorSyncStatus>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<StationHealth>,

    /// The current state of the management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<ManagementStationLifecycleState>,

    /// When enabled, the station setup script automatically runs to configure the firewall and SELinux settings on the station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auto_config_enabled: Option<bool>,

    /// The location of the instance that is acting as the management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<ManagedInstanceLocation>,

    /// Amount of available mirror storage in bytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror_storage_available_size: Option<i64>,

    /// Total mirror storage size in bytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror_storage_size: Option<i64>,

    /// The total size of all software source mirrors in bytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror_size: Option<i64>,

    /// The total number of unique packages within the mirrored software sources on the station. Each package is counted only once, regardless of how many versions it has. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror_unique_package_count: Option<i64>,

    /// The total number of all packages within the mirrored software sources. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror_package_count: Option<i64>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// System tags for this resource. Each key is predefined and scoped to a namespace. Example: {@code {\"orcl-cloud\": {\"free-tier-retained\": \"true\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Required fields for ManagementStation
pub struct ManagementStationRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the management station.
    pub compartment_id: String,

    /// A user-friendly name for the management station.
    pub display_name: String,

    /// Hostname of the management station.
    pub hostname: String,

    pub proxy: ProxyConfiguration,

    pub mirror: MirrorConfiguration,

    /// A list of other management stations that are behind the same load balancer within a high availability configuration. Stations are identified as peers if they have the same hostname and compartment.
    pub peer_management_stations: Vec<PeerManagementStation>,
}

impl ManagementStation {
    /// Create a new ManagementStation with required fields
    pub fn new(required: ManagementStationRequired) -> Self {
        Self {
            id: required.id,

            compartment_id: required.compartment_id,

            display_name: required.display_name,

            hostname: required.hostname,

            proxy: required.proxy,

            mirror: required.mirror,

            peer_management_stations: required.peer_management_stations,

            managed_instance_id: None,

            scheduled_job_id: None,

            profile_id: None,

            description: None,

            overall_state: None,

            overall_percentage: None,

            mirror_capacity: None,

            total_mirrors: None,

            mirror_sync_status: None,

            health: None,

            lifecycle_state: None,

            is_auto_config_enabled: None,

            location: None,

            mirror_storage_available_size: None,

            mirror_storage_size: None,

            mirror_size: None,

            mirror_unique_package_count: None,

            mirror_package_count: None,

            freeform_tags: None,

            defined_tags: None,

            system_tags: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set managed_instance_id
    pub fn set_managed_instance_id(mut self, value: Option<String>) -> Self {
        self.managed_instance_id = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set scheduled_job_id
    pub fn set_scheduled_job_id(mut self, value: Option<String>) -> Self {
        self.scheduled_job_id = value;
        self
    }

    /// Set profile_id
    pub fn set_profile_id(mut self, value: Option<String>) -> Self {
        self.profile_id = value;
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

    /// Set hostname
    pub fn set_hostname(mut self, value: String) -> Self {
        self.hostname = value;
        self
    }

    /// Set overall_state
    pub fn set_overall_state(mut self, value: Option<OverallState>) -> Self {
        self.overall_state = value;
        self
    }

    /// Set overall_percentage
    pub fn set_overall_percentage(mut self, value: Option<i64>) -> Self {
        self.overall_percentage = value;
        self
    }

    /// Set mirror_capacity
    pub fn set_mirror_capacity(mut self, value: Option<i64>) -> Self {
        self.mirror_capacity = value;
        self
    }

    /// Set total_mirrors
    pub fn set_total_mirrors(mut self, value: Option<i64>) -> Self {
        self.total_mirrors = value;
        self
    }

    /// Set mirror_sync_status
    pub fn set_mirror_sync_status(mut self, value: Option<MirrorSyncStatus>) -> Self {
        self.mirror_sync_status = value;
        self
    }

    /// Set proxy
    pub fn set_proxy(mut self, value: ProxyConfiguration) -> Self {
        self.proxy = value;
        self
    }

    /// Set mirror
    pub fn set_mirror(mut self, value: MirrorConfiguration) -> Self {
        self.mirror = value;
        self
    }

    /// Set health
    pub fn set_health(mut self, value: Option<StationHealth>) -> Self {
        self.health = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: Option<ManagementStationLifecycleState>) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set is_auto_config_enabled
    pub fn set_is_auto_config_enabled(mut self, value: Option<bool>) -> Self {
        self.is_auto_config_enabled = value;
        self
    }

    /// Set peer_management_stations
    pub fn set_peer_management_stations(mut self, value: Vec<PeerManagementStation>) -> Self {
        self.peer_management_stations = value;
        self
    }

    /// Set location
    pub fn set_location(mut self, value: Option<ManagedInstanceLocation>) -> Self {
        self.location = value;
        self
    }

    /// Set mirror_storage_available_size
    pub fn set_mirror_storage_available_size(mut self, value: Option<i64>) -> Self {
        self.mirror_storage_available_size = value;
        self
    }

    /// Set mirror_storage_size
    pub fn set_mirror_storage_size(mut self, value: Option<i64>) -> Self {
        self.mirror_storage_size = value;
        self
    }

    /// Set mirror_size
    pub fn set_mirror_size(mut self, value: Option<i64>) -> Self {
        self.mirror_size = value;
        self
    }

    /// Set mirror_unique_package_count
    pub fn set_mirror_unique_package_count(mut self, value: Option<i64>) -> Self {
        self.mirror_unique_package_count = value;
        self
    }

    /// Set mirror_package_count
    pub fn set_mirror_package_count(mut self, value: Option<i64>) -> Self {
        self.mirror_package_count = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.defined_tags = value;
        self
    }

    /// Set system_tags
    pub fn set_system_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.system_tags = value;
        self
    }

    /// Set managed_instance_id (unwraps Option)
    pub fn with_managed_instance_id(mut self, value: impl Into<String>) -> Self {
        self.managed_instance_id = Some(value.into());
        self
    }

    /// Set scheduled_job_id (unwraps Option)
    pub fn with_scheduled_job_id(mut self, value: impl Into<String>) -> Self {
        self.scheduled_job_id = Some(value.into());
        self
    }

    /// Set profile_id (unwraps Option)
    pub fn with_profile_id(mut self, value: impl Into<String>) -> Self {
        self.profile_id = Some(value.into());
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set overall_state (unwraps Option)
    pub fn with_overall_state(mut self, value: OverallState) -> Self {
        self.overall_state = Some(value);
        self
    }

    /// Set overall_percentage (unwraps Option)
    pub fn with_overall_percentage(mut self, value: i64) -> Self {
        self.overall_percentage = Some(value);
        self
    }

    /// Set mirror_capacity (unwraps Option)
    pub fn with_mirror_capacity(mut self, value: i64) -> Self {
        self.mirror_capacity = Some(value);
        self
    }

    /// Set total_mirrors (unwraps Option)
    pub fn with_total_mirrors(mut self, value: i64) -> Self {
        self.total_mirrors = Some(value);
        self
    }

    /// Set mirror_sync_status (unwraps Option)
    pub fn with_mirror_sync_status(mut self, value: MirrorSyncStatus) -> Self {
        self.mirror_sync_status = Some(value);
        self
    }

    /// Set health (unwraps Option)
    pub fn with_health(mut self, value: StationHealth) -> Self {
        self.health = Some(value);
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: ManagementStationLifecycleState) -> Self {
        self.lifecycle_state = Some(value);
        self
    }

    /// Set is_auto_config_enabled (unwraps Option)
    pub fn with_is_auto_config_enabled(mut self, value: bool) -> Self {
        self.is_auto_config_enabled = Some(value);
        self
    }

    /// Set location (unwraps Option)
    pub fn with_location(mut self, value: ManagedInstanceLocation) -> Self {
        self.location = Some(value);
        self
    }

    /// Set mirror_storage_available_size (unwraps Option)
    pub fn with_mirror_storage_available_size(mut self, value: i64) -> Self {
        self.mirror_storage_available_size = Some(value);
        self
    }

    /// Set mirror_storage_size (unwraps Option)
    pub fn with_mirror_storage_size(mut self, value: i64) -> Self {
        self.mirror_storage_size = Some(value);
        self
    }

    /// Set mirror_size (unwraps Option)
    pub fn with_mirror_size(mut self, value: i64) -> Self {
        self.mirror_size = Some(value);
        self
    }

    /// Set mirror_unique_package_count (unwraps Option)
    pub fn with_mirror_unique_package_count(mut self, value: i64) -> Self {
        self.mirror_unique_package_count = Some(value);
        self
    }

    /// Set mirror_package_count (unwraps Option)
    pub fn with_mirror_package_count(mut self, value: i64) -> Self {
        self.mirror_package_count = Some(value);
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set defined_tags (unwraps Option)
    pub fn with_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = Some(value);
        self
    }

    /// Set system_tags (unwraps Option)
    pub fn with_system_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.system_tags = Some(value);
        self
    }
}
