use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetachInstancePoolInstanceRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance pool.
    pub instance_pool_id: String,

    /// Instance being detached
    pub detach_instance_pool_instance_details: DetachInstancePoolInstanceDetails,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,
}

/// Required fields for DetachInstancePoolInstanceRequest
pub struct DetachInstancePoolInstanceRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance pool.
    pub instance_pool_id: String,

    /// Instance being detached
    pub detach_instance_pool_instance_details: DetachInstancePoolInstanceDetails,
}

impl DetachInstancePoolInstanceRequest {
    /// Create a new DetachInstancePoolInstanceRequest with required fields
    pub fn new(required: DetachInstancePoolInstanceRequestRequired) -> Self {
        Self {
            instance_pool_id: required.instance_pool_id,

            detach_instance_pool_instance_details: required.detach_instance_pool_instance_details,

            opc_retry_token: None,
        }
    }

    /// Set instance_pool_id
    pub fn set_instance_pool_id(mut self, value: String) -> Self {
        self.instance_pool_id = value;
        self
    }

    /// Set detach_instance_pool_instance_details
    pub fn set_detach_instance_pool_instance_details(
        mut self,
        value: DetachInstancePoolInstanceDetails,
    ) -> Self {
        self.detach_instance_pool_instance_details = value;
        self
    }

    /// Set opc_retry_token
    pub fn set_opc_retry_token(mut self, value: Option<String>) -> Self {
        self.opc_retry_token = value;
        self
    }

    /// Set opc_retry_token (unwraps Option)
    pub fn with_opc_retry_token(mut self, value: impl Into<String>) -> Self {
        self.opc_retry_token = Some(value.into());
        self
    }
}
