use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The customer facing GPU memory cluster instance object details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeGpuMemoryClusterInstanceSummary {
    /// The availability domain of the GPU memory cluster instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Customer-unique GPU memory cluster instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the compartment compartment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// The fault domain the GPU memory cluster instance is running in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    /// Configuration to be used for this GPU Memory Cluster instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_configuration_id: Option<String>,

    /// The region that contains the availability domain the instance is running in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// The shape of an instance. The shape determines the number of CPUs, amount of memory, and other resources allocated to the instance. The shape determines the number of CPUs, the amount of memory, and other resources allocated to the instance. You can list all available shapes by calling {@link #listShapes(ListShapesRequest) listShapes}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_shape: Option<String>,

    /// The lifecycle state of the GPU memory cluster instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<ComputeGpuMemoryClusterInstanceSummaryLifecycleState>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The date and time the GPU memory cluster instance was created. <p> Example: {@code 2016-09-15T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,
}

impl ComputeGpuMemoryClusterInstanceSummary {
    /// Create a new ComputeGpuMemoryClusterInstanceSummary
    pub fn new() -> Self {
        Self {
            availability_domain: None,

            id: None,

            compartment_id: None,

            fault_domain: None,

            instance_configuration_id: None,

            region: None,

            instance_shape: None,

            lifecycle_state: None,

            display_name: None,

            time_created: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: Option<String>) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: Option<String>) -> Self {
        self.id = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set fault_domain
    pub fn set_fault_domain(mut self, value: Option<String>) -> Self {
        self.fault_domain = value;
        self
    }

    /// Set instance_configuration_id
    pub fn set_instance_configuration_id(mut self, value: Option<String>) -> Self {
        self.instance_configuration_id = value;
        self
    }

    /// Set region
    pub fn set_region(mut self, value: Option<String>) -> Self {
        self.region = value;
        self
    }

    /// Set instance_shape
    pub fn set_instance_shape(mut self, value: Option<String>) -> Self {
        self.instance_shape = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(
        mut self,
        value: Option<ComputeGpuMemoryClusterInstanceSummaryLifecycleState>,
    ) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set availability_domain (unwraps Option)
    pub fn with_availability_domain(mut self, value: impl Into<String>) -> Self {
        self.availability_domain = Some(value.into());
        self
    }

    /// Set id (unwraps Option)
    pub fn with_id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
        self
    }

    /// Set fault_domain (unwraps Option)
    pub fn with_fault_domain(mut self, value: impl Into<String>) -> Self {
        self.fault_domain = Some(value.into());
        self
    }

    /// Set instance_configuration_id (unwraps Option)
    pub fn with_instance_configuration_id(mut self, value: impl Into<String>) -> Self {
        self.instance_configuration_id = Some(value.into());
        self
    }

    /// Set region (unwraps Option)
    pub fn with_region(mut self, value: impl Into<String>) -> Self {
        self.region = Some(value.into());
        self
    }

    /// Set instance_shape (unwraps Option)
    pub fn with_instance_shape(mut self, value: impl Into<String>) -> Self {
        self.instance_shape = Some(value.into());
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(
        mut self,
        value: ComputeGpuMemoryClusterInstanceSummaryLifecycleState,
    ) -> Self {
        self.lifecycle_state = Some(value);
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }
}

impl Default for ComputeGpuMemoryClusterInstanceSummary {
    fn default() -> Self {
        Self::new()
    }
}
