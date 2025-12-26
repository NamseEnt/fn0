use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Represents a load balancer that is to be attached to an instance pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachLoadBalancerDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the load balancer to attach to the instance pool.
    pub load_balancer_id: String,

    /// The name of the backend set on the load balancer to add instances to.
    pub backend_set_name: String,

    /// The port value to use when creating the backend set. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub port: i64,

    /// Indicates which VNIC on each instance in the pool should be used to associate with the load balancer. Possible values are \"PrimaryVnic\" or the displayName of one of the secondary VNICs on the instance configuration that is associated with the instance pool.
    pub vnic_selection: String,
}

/// Required fields for AttachLoadBalancerDetails
pub struct AttachLoadBalancerDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the load balancer to attach to the instance pool.
    pub load_balancer_id: String,

    /// The name of the backend set on the load balancer to add instances to.
    pub backend_set_name: String,

    /// The port value to use when creating the backend set. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub port: i64,

    /// Indicates which VNIC on each instance in the pool should be used to associate with the load balancer. Possible values are \"PrimaryVnic\" or the displayName of one of the secondary VNICs on the instance configuration that is associated with the instance pool.
    pub vnic_selection: String,
}

impl AttachLoadBalancerDetails {
    /// Create a new AttachLoadBalancerDetails with required fields
    pub fn new(required: AttachLoadBalancerDetailsRequired) -> Self {
        Self {
            load_balancer_id: required.load_balancer_id,

            backend_set_name: required.backend_set_name,

            port: required.port,

            vnic_selection: required.vnic_selection,
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

    /// Set port
    pub fn set_port(mut self, value: i64) -> Self {
        self.port = value;
        self
    }

    /// Set vnic_selection
    pub fn set_vnic_selection(mut self, value: String) -> Self {
        self.vnic_selection = value;
        self
    }
}
