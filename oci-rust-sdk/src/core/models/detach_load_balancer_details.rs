use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Represents a load balancer that is to be detached from an instance pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetachLoadBalancerDetails {
    /// The OCID of the load balancer to detach from the instance pool.
    pub load_balancer_id: String,

    /// The name of the backend set on the load balancer to detach from the instance pool.
    pub backend_set_name: String,
}

/// Required fields for DetachLoadBalancerDetails
pub struct DetachLoadBalancerDetailsRequired {
    /// The OCID of the load balancer to detach from the instance pool.
    pub load_balancer_id: String,

    /// The name of the backend set on the load balancer to detach from the instance pool.
    pub backend_set_name: String,
}

impl DetachLoadBalancerDetails {
    /// Create a new DetachLoadBalancerDetails with required fields
    pub fn new(required: DetachLoadBalancerDetailsRequired) -> Self {
        Self {
            load_balancer_id: required.load_balancer_id,

            backend_set_name: required.backend_set_name,
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
}
