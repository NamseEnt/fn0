use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Condensed instance data when listing instances on a dedicated VM host.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DedicatedVmHostInstanceSummary {
    /// The availability domain the virtual machine instance is running in. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment that contains the virtual machine instance.
    pub compartment_id: String,

    /// The OCID of the virtual machine instance.
    pub instance_id: String,

    /// The shape of the VM instance.
    pub shape: String,

    /// The date and time the virtual machine instance was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// Specifies whether the VM instance is confidential.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_memory_encryption_enabled: Option<bool>,
}

/// Required fields for DedicatedVmHostInstanceSummary
pub struct DedicatedVmHostInstanceSummaryRequired {
    /// The availability domain the virtual machine instance is running in. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment that contains the virtual machine instance.
    pub compartment_id: String,

    /// The OCID of the virtual machine instance.
    pub instance_id: String,

    /// The shape of the VM instance.
    pub shape: String,

    /// The date and time the virtual machine instance was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl DedicatedVmHostInstanceSummary {
    /// Create a new DedicatedVmHostInstanceSummary with required fields
    pub fn new(required: DedicatedVmHostInstanceSummaryRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            instance_id: required.instance_id,

            shape: required.shape,

            time_created: required.time_created,

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

    /// Set instance_id
    pub fn set_instance_id(mut self, value: String) -> Self {
        self.instance_id = value;
        self
    }

    /// Set is_memory_encryption_enabled
    pub fn set_is_memory_encryption_enabled(mut self, value: Option<bool>) -> Self {
        self.is_memory_encryption_enabled = value;
        self
    }

    /// Set shape
    pub fn set_shape(mut self, value: String) -> Self {
        self.shape = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set is_memory_encryption_enabled (unwraps Option)
    pub fn with_is_memory_encryption_enabled(mut self, value: bool) -> Self {
        self.is_memory_encryption_enabled = Some(value);
        self
    }
}
