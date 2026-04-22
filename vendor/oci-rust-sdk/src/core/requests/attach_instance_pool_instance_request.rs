use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachInstancePoolInstanceRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance pool.
    pub instance_pool_id: String,

    /// Attach an instance to a pool
    pub attach_instance_pool_instance_details: AttachInstancePoolInstanceDetails,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,
}

/// Required fields for AttachInstancePoolInstanceRequest
pub struct AttachInstancePoolInstanceRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance pool.
    pub instance_pool_id: String,

    /// Attach an instance to a pool
    pub attach_instance_pool_instance_details: AttachInstancePoolInstanceDetails,
}

impl AttachInstancePoolInstanceRequest {
    /// Create a new AttachInstancePoolInstanceRequest with required fields
    pub fn new(required: AttachInstancePoolInstanceRequestRequired) -> Self {
        Self {
            instance_pool_id: required.instance_pool_id,

            attach_instance_pool_instance_details: required.attach_instance_pool_instance_details,

            opc_retry_token: None,
        }
    }

    /// Set instance_pool_id
    pub fn set_instance_pool_id(mut self, value: String) -> Self {
        self.instance_pool_id = value;
        self
    }

    /// Set attach_instance_pool_instance_details
    pub fn set_attach_instance_pool_instance_details(
        mut self,
        value: AttachInstancePoolInstanceDetails,
    ) -> Self {
        self.attach_instance_pool_instance_details = value;
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
