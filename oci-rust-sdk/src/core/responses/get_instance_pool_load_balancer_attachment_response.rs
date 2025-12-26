use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInstancePoolLoadBalancerAttachmentResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.InstancePoolLoadBalancerAttachment instance.
    pub instance_pool_load_balancer_attachment: InstancePoolLoadBalancerAttachment,
}


/// Required fields for GetInstancePoolLoadBalancerAttachmentResponse
pub struct GetInstancePoolLoadBalancerAttachmentResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.InstancePoolLoadBalancerAttachment instance.
    pub instance_pool_load_balancer_attachment: InstancePoolLoadBalancerAttachment,
}

impl GetInstancePoolLoadBalancerAttachmentResponse {
    /// Create a new GetInstancePoolLoadBalancerAttachmentResponse with required fields
    pub fn new(required: GetInstancePoolLoadBalancerAttachmentResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            instance_pool_load_balancer_attachment: required.instance_pool_load_balancer_attachment,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set instance_pool_load_balancer_attachment
    pub fn set_instance_pool_load_balancer_attachment(mut self, value: InstancePoolLoadBalancerAttachment) -> Self {
        self.instance_pool_load_balancer_attachment = value;
        self
    }
}


