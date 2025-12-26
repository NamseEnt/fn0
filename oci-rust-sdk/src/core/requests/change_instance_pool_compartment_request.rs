use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeInstancePoolCompartmentRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance pool.
    pub instance_pool_id: String,

    /// Request to change the compartment of given instance pool.
    pub change_instance_pool_compartment_details: ChangeInstancePoolCompartmentDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,
}


/// Required fields for ChangeInstancePoolCompartmentRequest
pub struct ChangeInstancePoolCompartmentRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance pool.
    pub instance_pool_id: String,

    /// Request to change the compartment of given instance pool.
    pub change_instance_pool_compartment_details: ChangeInstancePoolCompartmentDetails,
}

impl ChangeInstancePoolCompartmentRequest {
    /// Create a new ChangeInstancePoolCompartmentRequest with required fields
    pub fn new(required: ChangeInstancePoolCompartmentRequestRequired) -> Self {
        Self {
            instance_pool_id: required.instance_pool_id,

            change_instance_pool_compartment_details: required.change_instance_pool_compartment_details,

            if_match: None,

            opc_request_id: None,

            opc_retry_token: None,
}
    }

    /// Set instance_pool_id
    pub fn set_instance_pool_id(mut self, value: String) -> Self {
        self.instance_pool_id = value;
        self
    }

    /// Set change_instance_pool_compartment_details
    pub fn set_change_instance_pool_compartment_details(mut self, value: ChangeInstancePoolCompartmentDetails) -> Self {
        self.change_instance_pool_compartment_details = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_retry_token
    pub fn set_opc_retry_token(mut self, value: Option<String>) -> Self {
        self.opc_retry_token = value;
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }

    /// Set opc_retry_token (unwraps Option)
    pub fn with_opc_retry_token(mut self, value: impl Into<String>) -> Self {
        self.opc_retry_token = Some(value.into());
        self
    }
}


