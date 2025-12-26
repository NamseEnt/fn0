use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// The customer facing object includes GPU memory fabric details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeGpuMemoryFabric {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Customer-unique GPU memory fabric
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the compartment. This should always be the root compartment.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for Customer-unique HPC Island
    pub compute_hpc_island_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for Customer-unique Network Block
    pub compute_network_block_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for Customer-unique Local Block
    pub compute_local_block_id: String,

    /// The lifecycle state of the GPU memory fabric
    pub lifecycle_state: ComputeGpuMemoryFabricLifecycleState,

    /// The health state of the GPU memory fabric
    pub fabric_health: ComputeGpuMemoryFabricFabricHealth,

    /// The total number of healthy bare metal hosts located in this compute GPU memory fabric. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub healthy_host_count: i64,

    /// The total number of bare metal hosts located in this compute GPU memory fabric. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub total_host_count: i64,

    /// The date and time that the compute GPU memory fabric record was created, in the format defined by [RFC3339] (https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// Additional data that can be exposed to the customer. Right now it will include the switch tray ids.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_data: Option<HashMap<String, serde_json::Value>>,

    /// The total number of available bare metal hosts located in this compute GPU memory fabric. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_host_count: Option<i64>,

    /// The host platform identifier used for bundle queries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_platform_name: Option<String>,

    /// The switch platform identifier used for bundle queries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switch_platform_name: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for current firmware bundle
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_firmware_bundle_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for targeted firmware bundle
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_firmware_bundle_id: Option<String>,

    /// The state of Memory Fabric Firmware update
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware_update_state: Option<ComputeGpuMemoryFabricFirmwareUpdateState>,

    /// The reason for updating firmware bundle version of the GPU memory fabric.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware_update_reason: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_fabric_preferences: Option<MemoryFabricPreferencesDescriptor>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Usage of system tag keys. These predefined keys are scoped to namespaces. Example: {@code { \"orcl-cloud\": { \"free-tier-retained\": \"true\" } }}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Required fields for ComputeGpuMemoryFabric
pub struct ComputeGpuMemoryFabricRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Customer-unique GPU memory fabric
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the compartment. This should always be the root compartment.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for Customer-unique HPC Island
    pub compute_hpc_island_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for Customer-unique Network Block
    pub compute_network_block_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for Customer-unique Local Block
    pub compute_local_block_id: String,

    /// The lifecycle state of the GPU memory fabric
    pub lifecycle_state: ComputeGpuMemoryFabricLifecycleState,

    /// The health state of the GPU memory fabric
    pub fabric_health: ComputeGpuMemoryFabricFabricHealth,

    /// The total number of healthy bare metal hosts located in this compute GPU memory fabric. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub healthy_host_count: i64,

    /// The total number of bare metal hosts located in this compute GPU memory fabric. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub total_host_count: i64,

    /// The date and time that the compute GPU memory fabric record was created, in the format defined by [RFC3339] (https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl ComputeGpuMemoryFabric {
    /// Create a new ComputeGpuMemoryFabric with required fields
    pub fn new(required: ComputeGpuMemoryFabricRequired) -> Self {
        Self {
            id: required.id,

            compartment_id: required.compartment_id,

            compute_hpc_island_id: required.compute_hpc_island_id,

            compute_network_block_id: required.compute_network_block_id,

            compute_local_block_id: required.compute_local_block_id,

            lifecycle_state: required.lifecycle_state,

            fabric_health: required.fabric_health,

            healthy_host_count: required.healthy_host_count,

            total_host_count: required.total_host_count,

            time_created: required.time_created,

            additional_data: None,

            available_host_count: None,

            host_platform_name: None,

            switch_platform_name: None,

            current_firmware_bundle_id: None,

            target_firmware_bundle_id: None,

            firmware_update_state: None,

            firmware_update_reason: None,

            memory_fabric_preferences: None,

            defined_tags: None,

            freeform_tags: None,

            system_tags: None,

            display_name: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set compute_hpc_island_id
    pub fn set_compute_hpc_island_id(mut self, value: String) -> Self {
        self.compute_hpc_island_id = value;
        self
    }

    /// Set compute_network_block_id
    pub fn set_compute_network_block_id(mut self, value: String) -> Self {
        self.compute_network_block_id = value;
        self
    }

    /// Set compute_local_block_id
    pub fn set_compute_local_block_id(mut self, value: String) -> Self {
        self.compute_local_block_id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: ComputeGpuMemoryFabricLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set fabric_health
    pub fn set_fabric_health(mut self, value: ComputeGpuMemoryFabricFabricHealth) -> Self {
        self.fabric_health = value;
        self
    }

    /// Set additional_data
    pub fn set_additional_data(
        mut self,
        value: Option<HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.additional_data = value;
        self
    }

    /// Set healthy_host_count
    pub fn set_healthy_host_count(mut self, value: i64) -> Self {
        self.healthy_host_count = value;
        self
    }

    /// Set available_host_count
    pub fn set_available_host_count(mut self, value: Option<i64>) -> Self {
        self.available_host_count = value;
        self
    }

    /// Set total_host_count
    pub fn set_total_host_count(mut self, value: i64) -> Self {
        self.total_host_count = value;
        self
    }

    /// Set host_platform_name
    pub fn set_host_platform_name(mut self, value: Option<String>) -> Self {
        self.host_platform_name = value;
        self
    }

    /// Set switch_platform_name
    pub fn set_switch_platform_name(mut self, value: Option<String>) -> Self {
        self.switch_platform_name = value;
        self
    }

    /// Set current_firmware_bundle_id
    pub fn set_current_firmware_bundle_id(mut self, value: Option<String>) -> Self {
        self.current_firmware_bundle_id = value;
        self
    }

    /// Set target_firmware_bundle_id
    pub fn set_target_firmware_bundle_id(mut self, value: Option<String>) -> Self {
        self.target_firmware_bundle_id = value;
        self
    }

    /// Set firmware_update_state
    pub fn set_firmware_update_state(
        mut self,
        value: Option<ComputeGpuMemoryFabricFirmwareUpdateState>,
    ) -> Self {
        self.firmware_update_state = value;
        self
    }

    /// Set firmware_update_reason
    pub fn set_firmware_update_reason(mut self, value: Option<String>) -> Self {
        self.firmware_update_reason = value;
        self
    }

    /// Set memory_fabric_preferences
    pub fn set_memory_fabric_preferences(
        mut self,
        value: Option<MemoryFabricPreferencesDescriptor>,
    ) -> Self {
        self.memory_fabric_preferences = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
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

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
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

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set additional_data (unwraps Option)
    pub fn with_additional_data(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.additional_data = Some(value);
        self
    }

    /// Set available_host_count (unwraps Option)
    pub fn with_available_host_count(mut self, value: i64) -> Self {
        self.available_host_count = Some(value);
        self
    }

    /// Set host_platform_name (unwraps Option)
    pub fn with_host_platform_name(mut self, value: impl Into<String>) -> Self {
        self.host_platform_name = Some(value.into());
        self
    }

    /// Set switch_platform_name (unwraps Option)
    pub fn with_switch_platform_name(mut self, value: impl Into<String>) -> Self {
        self.switch_platform_name = Some(value.into());
        self
    }

    /// Set current_firmware_bundle_id (unwraps Option)
    pub fn with_current_firmware_bundle_id(mut self, value: impl Into<String>) -> Self {
        self.current_firmware_bundle_id = Some(value.into());
        self
    }

    /// Set target_firmware_bundle_id (unwraps Option)
    pub fn with_target_firmware_bundle_id(mut self, value: impl Into<String>) -> Self {
        self.target_firmware_bundle_id = Some(value.into());
        self
    }

    /// Set firmware_update_state (unwraps Option)
    pub fn with_firmware_update_state(
        mut self,
        value: ComputeGpuMemoryFabricFirmwareUpdateState,
    ) -> Self {
        self.firmware_update_state = Some(value);
        self
    }

    /// Set firmware_update_reason (unwraps Option)
    pub fn with_firmware_update_reason(mut self, value: impl Into<String>) -> Self {
        self.firmware_update_reason = Some(value.into());
        self
    }

    /// Set memory_fabric_preferences (unwraps Option)
    pub fn with_memory_fabric_preferences(
        mut self,
        value: MemoryFabricPreferencesDescriptor,
    ) -> Self {
        self.memory_fabric_preferences = Some(value);
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

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
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

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }
}
