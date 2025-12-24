use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A dedicated virtual machine host lets you host multiple VM instances on a dedicated server that is not shared with other tenancies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DedicatedVmHost {
    /// The availability domain the dedicated virtual machine host is running in. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment that contains the dedicated virtual machine host.
    pub compartment_id: String,

    /// The dedicated virtual machine host shape. The shape determines the number of CPUs and other resources available for VMs.
    pub dedicated_vm_host_shape: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the dedicated VM host.
    pub id: String,

    /// The current state of the dedicated VM host.
    pub lifecycle_state: DedicatedVmHostLifecycleState,

    /// The date and time the dedicated VM host was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The total OCPUs of the dedicated VM host. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub total_ocpus: i64,

    /// The available OCPUs of the dedicated VM host. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub remaining_ocpus: i64,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// The fault domain for the dedicated virtual machine host's assigned instances. For more information, see [Fault Domains](https://docs.oracle.com/iaas/Content/General/Concepts/regions.htm#fault). <p> If you do not specify the fault domain, the system selects one for you. To change the fault domain for a dedicated virtual machine host, delete it, and then create a new dedicated virtual machine host in the preferred fault domain. <p> To get a list of fault domains, use the {@code ListFaultDomains} operation in the [Identity and Access Management Service API](https://docs.oracle.com/iaas/api/#/en/identity/20160918/). <p> Example: {@code FAULT-DOMAIN-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement_constraint_details: Option<HostGroupPlacementConstraintDetails>,

    /// The capacity configuration selected to be configured for the Dedicated Virtual Machine host. Run {@link #listDedicatedVmHostShapes(ListDedicatedVmHostShapesRequest) listDedicatedVmHostShapes} API to see details of this capacity configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_config: Option<String>,

    /// Specifies if the Dedicated Virtual Machine Host (DVMH) is restricted to running only Confidential VMs. If {@code true}, only Confidential VMs can be launched. If {@code false}, Confidential VMs cannot be launched.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_memory_encryption_enabled: Option<bool>,

    /// The total memory of the dedicated VM host, in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_memory_in_g_bs: Option<i64>,

    /// The remaining memory of the dedicated VM host, in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_memory_in_g_bs: Option<i64>,

    /// A list of total and remaining CPU and memory per capacity bucket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_bins: Option<Vec<CapacityBin>>,

    /// The compute bare metal host OCID of the dedicated virtual machine host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_bare_metal_host_id: Option<String>,
}

/// Required fields for DedicatedVmHost
pub struct DedicatedVmHostRequired {
    /// The availability domain the dedicated virtual machine host is running in. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment that contains the dedicated virtual machine host.
    pub compartment_id: String,

    /// The dedicated virtual machine host shape. The shape determines the number of CPUs and other resources available for VMs.
    pub dedicated_vm_host_shape: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the dedicated VM host.
    pub id: String,

    /// The current state of the dedicated VM host.
    pub lifecycle_state: DedicatedVmHostLifecycleState,

    /// The date and time the dedicated VM host was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The total OCPUs of the dedicated VM host. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub total_ocpus: i64,

    /// The available OCPUs of the dedicated VM host. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub remaining_ocpus: i64,
}

impl DedicatedVmHost {
    /// Create a new DedicatedVmHost with required fields
    pub fn new(required: DedicatedVmHostRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            dedicated_vm_host_shape: required.dedicated_vm_host_shape,

            display_name: required.display_name,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            time_created: required.time_created,

            total_ocpus: required.total_ocpus,

            remaining_ocpus: required.remaining_ocpus,

            defined_tags: None,

            fault_domain: None,

            freeform_tags: None,

            placement_constraint_details: None,

            capacity_config: None,

            is_memory_encryption_enabled: None,

            total_memory_in_g_bs: None,

            remaining_memory_in_g_bs: None,

            capacity_bins: None,

            compute_bare_metal_host_id: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set dedicated_vm_host_shape
    pub fn set_dedicated_vm_host_shape(mut self, value: String) -> Self {
        self.dedicated_vm_host_shape = value;
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

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set fault_domain
    pub fn set_fault_domain(mut self, value: Option<String>) -> Self {
        self.fault_domain = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set placement_constraint_details
    pub fn set_placement_constraint_details(
        mut self,
        value: Option<HostGroupPlacementConstraintDetails>,
    ) -> Self {
        self.placement_constraint_details = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set capacity_config
    pub fn set_capacity_config(mut self, value: Option<String>) -> Self {
        self.capacity_config = value;
        self
    }

    /// Set is_memory_encryption_enabled
    pub fn set_is_memory_encryption_enabled(mut self, value: Option<bool>) -> Self {
        self.is_memory_encryption_enabled = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: DedicatedVmHostLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set total_ocpus
    pub fn set_total_ocpus(mut self, value: i64) -> Self {
        self.total_ocpus = value;
        self
    }

    /// Set remaining_ocpus
    pub fn set_remaining_ocpus(mut self, value: i64) -> Self {
        self.remaining_ocpus = value;
        self
    }

    /// Set total_memory_in_g_bs
    pub fn set_total_memory_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.total_memory_in_g_bs = value;
        self
    }

    /// Set remaining_memory_in_g_bs
    pub fn set_remaining_memory_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.remaining_memory_in_g_bs = value;
        self
    }

    /// Set capacity_bins
    pub fn set_capacity_bins(mut self, value: Option<Vec<CapacityBin>>) -> Self {
        self.capacity_bins = value;
        self
    }

    /// Set compute_bare_metal_host_id
    pub fn set_compute_bare_metal_host_id(mut self, value: Option<String>) -> Self {
        self.compute_bare_metal_host_id = value;
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

    /// Set fault_domain (unwraps Option)
    pub fn with_fault_domain(mut self, value: impl Into<String>) -> Self {
        self.fault_domain = Some(value.into());
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set placement_constraint_details (unwraps Option)
    pub fn with_placement_constraint_details(
        mut self,
        value: HostGroupPlacementConstraintDetails,
    ) -> Self {
        self.placement_constraint_details = Some(value);
        self
    }

    /// Set capacity_config (unwraps Option)
    pub fn with_capacity_config(mut self, value: impl Into<String>) -> Self {
        self.capacity_config = Some(value.into());
        self
    }

    /// Set is_memory_encryption_enabled (unwraps Option)
    pub fn with_is_memory_encryption_enabled(mut self, value: bool) -> Self {
        self.is_memory_encryption_enabled = Some(value);
        self
    }

    /// Set total_memory_in_g_bs (unwraps Option)
    pub fn with_total_memory_in_g_bs(mut self, value: i64) -> Self {
        self.total_memory_in_g_bs = Some(value);
        self
    }

    /// Set remaining_memory_in_g_bs (unwraps Option)
    pub fn with_remaining_memory_in_g_bs(mut self, value: i64) -> Self {
        self.remaining_memory_in_g_bs = Some(value);
        self
    }

    /// Set capacity_bins (unwraps Option)
    pub fn with_capacity_bins(mut self, value: Vec<CapacityBin>) -> Self {
        self.capacity_bins = Some(value);
        self
    }

    /// Set compute_bare_metal_host_id (unwraps Option)
    pub fn with_compute_bare_metal_host_id(mut self, value: impl Into<String>) -> Self {
        self.compute_bare_metal_host_id = Some(value.into());
        self
    }
}
