use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A dedicated virtual machine (VM) host lets you host multiple instances on a dedicated server that is not shared with other tenancies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DedicatedVmHostSummary {
    /// The availability domain the dedicated VM host is running in. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment that contains the dedicated VM host.
    pub compartment_id: String,

    /// The shape of the dedicated VM host. The shape determines the number of CPUs and other resources available for VMs.
    pub dedicated_vm_host_shape: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the dedicated VM host.
    pub id: String,

    /// The current state of the dedicated VM host.
    pub lifecycle_state: DedicatedVmHostSummaryLifecycleState,

    /// The date and time the dedicated VM host was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The current available OCPUs of the dedicated VM host. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub remaining_ocpus: i64,

    /// The current total OCPUs of the dedicated VM host. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub total_ocpus: i64,

    /// The fault domain for the dedicated VM host's assigned instances. For more information, see Fault Domains. <p> If you do not specify the fault domain, the system selects one for you. To change the fault domain for a dedicated VM host, delete it and create a new dedicated VM host in the preferred fault domain. <p> To get a list of fault domains, use the ListFaultDomains operation in the Identity and Access Management Service API. <p> Example: {@code FAULT-DOMAIN-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    /// Specifies if the Dedicated Virtual Machine Host is restricted to running only Confidential VMs. If {@code true}, only Confidential VMs can be launched. If {@code false}, Confidential VMs cannot be launched.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_memory_encryption_enabled: Option<bool>,

    /// The current total memory of the dedicated VM host, in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_memory_in_g_bs: Option<i64>,

    /// The current available memory of the dedicated VM host, in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_memory_in_g_bs: Option<i64>,
}

/// Required fields for DedicatedVmHostSummary
pub struct DedicatedVmHostSummaryRequired {
    /// The availability domain the dedicated VM host is running in. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment that contains the dedicated VM host.
    pub compartment_id: String,

    /// The shape of the dedicated VM host. The shape determines the number of CPUs and other resources available for VMs.
    pub dedicated_vm_host_shape: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the dedicated VM host.
    pub id: String,

    /// The current state of the dedicated VM host.
    pub lifecycle_state: DedicatedVmHostSummaryLifecycleState,

    /// The date and time the dedicated VM host was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The current available OCPUs of the dedicated VM host. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub remaining_ocpus: i64,

    /// The current total OCPUs of the dedicated VM host. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub total_ocpus: i64,
}

impl DedicatedVmHostSummary {
    /// Create a new DedicatedVmHostSummary with required fields
    pub fn new(required: DedicatedVmHostSummaryRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            dedicated_vm_host_shape: required.dedicated_vm_host_shape,

            display_name: required.display_name,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            time_created: required.time_created,

            remaining_ocpus: required.remaining_ocpus,

            total_ocpus: required.total_ocpus,

            fault_domain: None,

            is_memory_encryption_enabled: None,

            total_memory_in_g_bs: None,

            remaining_memory_in_g_bs: None,
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

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set is_memory_encryption_enabled
    pub fn set_is_memory_encryption_enabled(mut self, value: Option<bool>) -> Self {
        self.is_memory_encryption_enabled = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: DedicatedVmHostSummaryLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set remaining_ocpus
    pub fn set_remaining_ocpus(mut self, value: i64) -> Self {
        self.remaining_ocpus = value;
        self
    }

    /// Set total_ocpus
    pub fn set_total_ocpus(mut self, value: i64) -> Self {
        self.total_ocpus = value;
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

    /// Set fault_domain (unwraps Option)
    pub fn with_fault_domain(mut self, value: impl Into<String>) -> Self {
        self.fault_domain = Some(value.into());
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
}
