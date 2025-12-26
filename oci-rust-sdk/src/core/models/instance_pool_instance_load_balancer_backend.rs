use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Represents the load balancer Backend that is configured for an instance pool instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstancePoolInstanceLoadBalancerBackend {
    /// The OCID of the load balancer attached to the instance pool.
    pub load_balancer_id: String,

    /// The name of the backend set on the load balancer.
    pub backend_set_name: String,

    /// The name of the backend in the backend set.
    pub backend_name: String,

    /// The health of the backend as observed by the load balancer.
    pub backend_health_status: InstancePoolInstanceLoadBalancerBackendBackendHealthStatus,
}

/// Required fields for InstancePoolInstanceLoadBalancerBackend
pub struct InstancePoolInstanceLoadBalancerBackendRequired {
    /// The OCID of the load balancer attached to the instance pool.
    pub load_balancer_id: String,

    /// The name of the backend set on the load balancer.
    pub backend_set_name: String,

    /// The name of the backend in the backend set.
    pub backend_name: String,

    /// The health of the backend as observed by the load balancer.
    pub backend_health_status: InstancePoolInstanceLoadBalancerBackendBackendHealthStatus,
}

impl InstancePoolInstanceLoadBalancerBackend {
    /// Create a new InstancePoolInstanceLoadBalancerBackend with required fields
    pub fn new(required: InstancePoolInstanceLoadBalancerBackendRequired) -> Self {
        Self {
            load_balancer_id: required.load_balancer_id,

            backend_set_name: required.backend_set_name,

            backend_name: required.backend_name,

            backend_health_status: required.backend_health_status,
        }
    }

    /// Set load_balancer_id
    pub fn set_load_balancer_id(mut self, value: String) -> Self {
        self.load_balancer_id = value;
        self
    }

    /// Set backend_set_name
    pub fn set_backend_set_name(mut self, value: String) -> Self {
        self.backend_set_name = value;
        self
    }

    /// Set backend_name
    pub fn set_backend_name(mut self, value: String) -> Self {
        self.backend_name = value;
        self
    }

    /// Set backend_health_status
    pub fn set_backend_health_status(
        mut self,
        value: InstancePoolInstanceLoadBalancerBackendBackendHealthStatus,
    ) -> Self {
        self.backend_health_status = value;
        self
    }
}
