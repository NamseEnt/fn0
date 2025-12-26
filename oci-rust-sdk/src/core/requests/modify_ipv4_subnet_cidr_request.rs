use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifyIpv4SubnetCidrRequest {
    /// Specify the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet.
    pub subnet_id: String,

    /// Details object for updating a SUBNET IPv4 prefix.
    pub modify_ipv4_subnet_cidr_details: ModifyIpv4SubnetCidrDetails,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for ModifyIpv4SubnetCidrRequest
pub struct ModifyIpv4SubnetCidrRequestRequired {
    /// Specify the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet.
    pub subnet_id: String,

    /// Details object for updating a SUBNET IPv4 prefix.
    pub modify_ipv4_subnet_cidr_details: ModifyIpv4SubnetCidrDetails,
}

impl ModifyIpv4SubnetCidrRequest {
    /// Create a new ModifyIpv4SubnetCidrRequest with required fields
    pub fn new(required: ModifyIpv4SubnetCidrRequestRequired) -> Self {
        Self {
            subnet_id: required.subnet_id,

            modify_ipv4_subnet_cidr_details: required.modify_ipv4_subnet_cidr_details,

            opc_retry_token: None,

            if_match: None,

            opc_request_id: None,
}
    }

    /// Set subnet_id
    pub fn set_subnet_id(mut self, value: String) -> Self {
        self.subnet_id = value;
        self
    }

    /// Set modify_ipv4_subnet_cidr_details
    pub fn set_modify_ipv4_subnet_cidr_details(mut self, value: ModifyIpv4SubnetCidrDetails) -> Self {
        self.modify_ipv4_subnet_cidr_details = value;
        self
    }

    /// Set opc_retry_token
    pub fn set_opc_retry_token(mut self, value: Option<String>) -> Self {
        self.opc_retry_token = value;
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

    /// Set opc_retry_token (unwraps Option)
    pub fn with_opc_retry_token(mut self, value: impl Into<String>) -> Self {
        self.opc_retry_token = Some(value.into());
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
}


