use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInstancePoolLoadBalancerAttachmentRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance pool.
    pub instance_pool_id: String,

    /// The OCID of the load balancer attachment.
    pub instance_pool_load_balancer_attachment_id: String,
}

/// Required fields for GetInstancePoolLoadBalancerAttachmentRequest
pub struct GetInstancePoolLoadBalancerAttachmentRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance pool.
    pub instance_pool_id: String,

    /// The OCID of the load balancer attachment.
    pub instance_pool_load_balancer_attachment_id: String,
}

impl GetInstancePoolLoadBalancerAttachmentRequest {
    /// Create a new GetInstancePoolLoadBalancerAttachmentRequest with required fields
    pub fn new(required: GetInstancePoolLoadBalancerAttachmentRequestRequired) -> Self {
        Self {
            instance_pool_id: required.instance_pool_id,

            instance_pool_load_balancer_attachment_id: required
                .instance_pool_load_balancer_attachment_id,
        }
    }

    /// Set instance_pool_id
    pub fn set_instance_pool_id(mut self, value: String) -> Self {
        self.instance_pool_id = value;
        self
    }

    /// Set instance_pool_load_balancer_attachment_id
    pub fn set_instance_pool_load_balancer_attachment_id(mut self, value: String) -> Self {
        self.instance_pool_load_balancer_attachment_id = value;
        self
    }
}
