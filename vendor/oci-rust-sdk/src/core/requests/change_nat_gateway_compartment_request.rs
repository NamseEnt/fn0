use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeNatGatewayCompartmentRequest {
    /// The NAT gateway's [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub nat_gateway_id: String,

    /// Request to change the compartment of a given NAT Gateway.
    pub change_nat_gateway_compartment_details: ChangeNatGatewayCompartmentDetails,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,
}

/// Required fields for ChangeNatGatewayCompartmentRequest
pub struct ChangeNatGatewayCompartmentRequestRequired {
    /// The NAT gateway's [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub nat_gateway_id: String,

    /// Request to change the compartment of a given NAT Gateway.
    pub change_nat_gateway_compartment_details: ChangeNatGatewayCompartmentDetails,
}

impl ChangeNatGatewayCompartmentRequest {
    /// Create a new ChangeNatGatewayCompartmentRequest with required fields
    pub fn new(required: ChangeNatGatewayCompartmentRequestRequired) -> Self {
        Self {
            nat_gateway_id: required.nat_gateway_id,

            change_nat_gateway_compartment_details: required.change_nat_gateway_compartment_details,

            opc_request_id: None,

            opc_retry_token: None,
        }
    }

    /// Set nat_gateway_id
    pub fn set_nat_gateway_id(mut self, value: String) -> Self {
        self.nat_gateway_id = value;
        self
    }

    /// Set change_nat_gateway_compartment_details
    pub fn set_change_nat_gateway_compartment_details(
        mut self,
        value: ChangeNatGatewayCompartmentDetails,
    ) -> Self {
        self.change_nat_gateway_compartment_details = value;
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
