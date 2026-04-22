use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A public IP prefix and its details. With a public virtual circuit, the customer specifies the customer-owned public IP prefixes to advertise across the connection. For more information, see [FastConnect Overview](https://docs.oracle.com/iaas/Content/Network/Concepts/fastconnect.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualCircuitPublicPrefix {
    /// Publix IP prefix (CIDR) that the customer specified.
    pub cidr_block: String,

    /// Oracle must verify that the customer owns the public IP prefix before traffic for that prefix can flow across the virtual circuit. Verification can take a few business days. {@code IN_PROGRESS} means Oracle is verifying the prefix. {@code COMPLETED} means verification succeeded. {@code FAILED} means verification failed and traffic for this prefix will not flow across the connection.
    pub verification_state: VirtualCircuitPublicPrefixVerificationState,
}

/// Required fields for VirtualCircuitPublicPrefix
pub struct VirtualCircuitPublicPrefixRequired {
    /// Publix IP prefix (CIDR) that the customer specified.
    pub cidr_block: String,

    /// Oracle must verify that the customer owns the public IP prefix before traffic for that prefix can flow across the virtual circuit. Verification can take a few business days. {@code IN_PROGRESS} means Oracle is verifying the prefix. {@code COMPLETED} means verification succeeded. {@code FAILED} means verification failed and traffic for this prefix will not flow across the connection.
    pub verification_state: VirtualCircuitPublicPrefixVerificationState,
}

impl VirtualCircuitPublicPrefix {
    /// Create a new VirtualCircuitPublicPrefix with required fields
    pub fn new(required: VirtualCircuitPublicPrefixRequired) -> Self {
        Self {
            cidr_block: required.cidr_block,

            verification_state: required.verification_state,
        }
    }

    /// Set cidr_block
    pub fn set_cidr_block(mut self, value: String) -> Self {
        self.cidr_block = value;
        self
    }

    /// Set verification_state
    pub fn set_verification_state(
        mut self,
        value: VirtualCircuitPublicPrefixVerificationState,
    ) -> Self {
        self.verification_state = value;
        self
    }
}
