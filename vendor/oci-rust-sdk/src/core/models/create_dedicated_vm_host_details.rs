use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// The details for creating a new dedicated virtual machine host.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDedicatedVmHostDetails {
    /// The availability domain of the dedicated virtual machine host. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment.
    pub compartment_id: String,

    /// The dedicated virtual machine host shape. The shape determines the number of CPUs and other resources available for VM instances launched on the dedicated virtual machine host.
    pub dedicated_vm_host_shape: String,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The fault domain for the dedicated virtual machine host's assigned instances. For more information, see [Fault Domains](https://docs.oracle.com/iaas/Content/General/Concepts/regions.htm#fault). If you do not specify the fault domain, the system selects one for you. To change the fault domain for a dedicated virtual machine host, delete it and create a new dedicated virtual machine host in the preferred fault domain. <p> To get a list of fault domains, use the {@code ListFaultDomains} operation in the [Identity and Access Management Service API](https://docs.oracle.com/iaas/api/#/en/identity/20160918/). <p> Example: {@code FAULT-DOMAIN-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement_constraint_details: Option<HostGroupPlacementConstraintDetails>,

    /// The capacity configuration selected to be configured for the Dedicated Virtual Machine host. Run {@link #listDedicatedVmHostShapes(ListDedicatedVmHostShapesRequest) listDedicatedVmHostShapes} API first to see the capacity configuration options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_config: Option<String>,

    /// Specifies if the Dedicated Virtual Machine Host (DVMH) is restricted to running only Confidential VMs. If {@code true}, only Confidential VMs can be launched. If {@code false}, Confidential VMs cannot be launched.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_memory_encryption_enabled: Option<bool>,
}

/// Required fields for CreateDedicatedVmHostDetails
pub struct CreateDedicatedVmHostDetailsRequired {
    /// The availability domain of the dedicated virtual machine host. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment.
    pub compartment_id: String,

    /// The dedicated virtual machine host shape. The shape determines the number of CPUs and other resources available for VM instances launched on the dedicated virtual machine host.
    pub dedicated_vm_host_shape: String,
}

impl CreateDedicatedVmHostDetails {
    /// Create a new CreateDedicatedVmHostDetails with required fields
    pub fn new(required: CreateDedicatedVmHostDetailsRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            dedicated_vm_host_shape: required.dedicated_vm_host_shape,

            defined_tags: None,

            display_name: None,

            fault_domain: None,

            freeform_tags: None,

            placement_constraint_details: None,

            capacity_config: None,

            is_memory_encryption_enabled: None,
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
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
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

    /// Set defined_tags (unwraps Option)
    pub fn with_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = Some(value);
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
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
}
