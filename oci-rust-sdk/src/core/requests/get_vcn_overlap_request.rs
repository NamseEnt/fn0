use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVcnOverlapRequest {
    /// Specify the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN.
    pub vcn_id: String,

    /// Lists details of the IP Inventory VCN overlap data.
    pub get_vcn_overlap_details: GetIpInventoryVcnOverlapDetails,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,
}

/// Required fields for GetVcnOverlapRequest
pub struct GetVcnOverlapRequestRequired {
    /// Specify the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN.
    pub vcn_id: String,

    /// Lists details of the IP Inventory VCN overlap data.
    pub get_vcn_overlap_details: GetIpInventoryVcnOverlapDetails,
}

impl GetVcnOverlapRequest {
    /// Create a new GetVcnOverlapRequest with required fields
    pub fn new(required: GetVcnOverlapRequestRequired) -> Self {
        Self {
            vcn_id: required.vcn_id,

            get_vcn_overlap_details: required.get_vcn_overlap_details,

            opc_request_id: None,

            opc_retry_token: None,
        }
    }

    /// Set vcn_id
    pub fn set_vcn_id(mut self, value: String) -> Self {
        self.vcn_id = value;
        self
    }

    /// Set get_vcn_overlap_details
    pub fn set_get_vcn_overlap_details(mut self, value: GetIpInventoryVcnOverlapDetails) -> Self {
        self.get_vcn_overlap_details = value;
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
