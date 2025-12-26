use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Represents a load balancer that is attached to an instance pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstancePoolLoadBalancerAttachment {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the load balancer attachment.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance pool of the load balancer attachment.
    pub instance_pool_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the load balancer attached to the instance pool.
    pub load_balancer_id: String,

    /// The name of the backend set on the load balancer.
    pub backend_set_name: String,

    /// The port value used for the backends. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub port: i64,

    /// Indicates which VNIC on each instance in the instance pool should be used to associate with the load balancer. Possible values are \"PrimaryVnic\" or the displayName of one of the secondary VNICs on the instance configuration that is associated with the instance pool.
    pub vnic_selection: String,

    /// The status of the interaction between the instance pool and the load balancer.
    pub lifecycle_state: InstancePoolLoadBalancerAttachmentLifecycleState,
}

/// Required fields for InstancePoolLoadBalancerAttachment
pub struct InstancePoolLoadBalancerAttachmentRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the load balancer attachment.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance pool of the load balancer attachment.
    pub instance_pool_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the load balancer attached to the instance pool.
    pub load_balancer_id: String,

    /// The name of the backend set on the load balancer.
    pub backend_set_name: String,

    /// The port value used for the backends. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub port: i64,

    /// Indicates which VNIC on each instance in the instance pool should be used to associate with the load balancer. Possible values are \"PrimaryVnic\" or the displayName of one of the secondary VNICs on the instance configuration that is associated with the instance pool.
    pub vnic_selection: String,

    /// The status of the interaction between the instance pool and the load balancer.
    pub lifecycle_state: InstancePoolLoadBalancerAttachmentLifecycleState,
}

impl InstancePoolLoadBalancerAttachment {
    /// Create a new InstancePoolLoadBalancerAttachment with required fields
    pub fn new(required: InstancePoolLoadBalancerAttachmentRequired) -> Self {
        Self {
            id: required.id,

            instance_pool_id: required.instance_pool_id,

            load_balancer_id: required.load_balancer_id,

            backend_set_name: required.backend_set_name,

            port: required.port,

            vnic_selection: required.vnic_selection,

            lifecycle_state: required.lifecycle_state,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set instance_pool_id
    pub fn set_instance_pool_id(mut self, value: String) -> Self {
        self.instance_pool_id = value;
        self
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

    /// Set lifecycle_state
    pub fn set_lifecycle_state(
        mut self,
        value: InstancePoolLoadBalancerAttachmentLifecycleState,
    ) -> Self {
        self.lifecycle_state = value;
        self
    }
}
