use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachServiceIdRequest {
    /// The service gateway's [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub service_gateway_id: String,

    /// ServiceId of Service to be attached to a service gateway.
    pub attach_service_details: ServiceIdRequestDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for AttachServiceIdRequest
pub struct AttachServiceIdRequestRequired {
    /// The service gateway's [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub service_gateway_id: String,

    /// ServiceId of Service to be attached to a service gateway.
    pub attach_service_details: ServiceIdRequestDetails,
}

impl AttachServiceIdRequest {
    /// Create a new AttachServiceIdRequest with required fields
    pub fn new(required: AttachServiceIdRequestRequired) -> Self {
        Self {
            service_gateway_id: required.service_gateway_id,

            attach_service_details: required.attach_service_details,

            if_match: None,
}
    }

    /// Set service_gateway_id
    pub fn set_service_gateway_id(mut self, value: String) -> Self {
        self.service_gateway_id = value;
        self
    }

    /// Set attach_service_details
    pub fn set_attach_service_details(mut self, value: ServiceIdRequestDetails) -> Self {
        self.attach_service_details = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }
}


