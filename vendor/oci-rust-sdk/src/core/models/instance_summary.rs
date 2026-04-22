use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Condensed instance data when listing instances in an instance pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSummary {
    /// The OCID of the instance.
    pub id: String,

    /// The availability domain the instance is running in.
    pub availability_domain: String,

    /// The OCID of the compartment that contains the instance.
    pub compartment_id: String,

    /// The OCID of the instance confgiuration used to create the instance.
    pub instance_configuration_id: String,

    /// The region that contains the availability domain the instance is running in.
    pub region: String,

    /// The current state of the instance pool instance.
    pub state: String,

    /// The date and time the instance pool instance was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The fault domain the instance is running in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    /// The shape of an instance. The shape determines the number of CPUs, amount of memory, and other resources allocated to the instance. <p> You can enumerate all available shapes by calling {@link #listShapes(ListShapesRequest) listShapes}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,

    /// The load balancer backends that are configured for the instance pool instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_balancer_backends: Option<Vec<InstancePoolInstanceLoadBalancerBackend>>,
}

/// Required fields for InstanceSummary
pub struct InstanceSummaryRequired {
    /// The OCID of the instance.
    pub id: String,

    /// The availability domain the instance is running in.
    pub availability_domain: String,

    /// The OCID of the compartment that contains the instance.
    pub compartment_id: String,

    /// The OCID of the instance confgiuration used to create the instance.
    pub instance_configuration_id: String,

    /// The region that contains the availability domain the instance is running in.
    pub region: String,

    /// The current state of the instance pool instance.
    pub state: String,

    /// The date and time the instance pool instance was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl InstanceSummary {
    /// Create a new InstanceSummary with required fields
    pub fn new(required: InstanceSummaryRequired) -> Self {
        Self {
            id: required.id,

            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            instance_configuration_id: required.instance_configuration_id,

            region: required.region,

            state: required.state,

            time_created: required.time_created,

            display_name: None,

            fault_domain: None,

            shape: None,

            load_balancer_backends: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
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

    /// Set instance_configuration_id
    pub fn set_instance_configuration_id(mut self, value: String) -> Self {
        self.instance_configuration_id = value;
        self
    }

    /// Set region
    pub fn set_region(mut self, value: String) -> Self {
        self.region = value;
        self
    }

    /// Set shape
    pub fn set_shape(mut self, value: Option<String>) -> Self {
        self.shape = value;
        self
    }

    /// Set state
    pub fn set_state(mut self, value: String) -> Self {
        self.state = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set load_balancer_backends
    pub fn set_load_balancer_backends(
        mut self,
        value: Option<Vec<InstancePoolInstanceLoadBalancerBackend>>,
    ) -> Self {
        self.load_balancer_backends = value;
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

    /// Set shape (unwraps Option)
    pub fn with_shape(mut self, value: impl Into<String>) -> Self {
        self.shape = Some(value.into());
        self
    }

    /// Set load_balancer_backends (unwraps Option)
    pub fn with_load_balancer_backends(
        mut self,
        value: Vec<InstancePoolInstanceLoadBalancerBackend>,
    ) -> Self {
        self.load_balancer_backends = Some(value);
        self
    }
}
