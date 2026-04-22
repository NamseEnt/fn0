use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstanceConfigurationRequest {
    /// Instance configuration creation details
    pub create_instance_configuration: CreateInstanceConfigurationDetails,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,
}

/// Required fields for CreateInstanceConfigurationRequest
pub struct CreateInstanceConfigurationRequestRequired {
    /// Instance configuration creation details
    pub create_instance_configuration: CreateInstanceConfigurationDetails,
}

impl CreateInstanceConfigurationRequest {
    /// Create a new CreateInstanceConfigurationRequest with required fields
    pub fn new(required: CreateInstanceConfigurationRequestRequired) -> Self {
        Self {
            create_instance_configuration: required.create_instance_configuration,

            opc_retry_token: None,
        }
    }

    /// Set create_instance_configuration
    pub fn set_create_instance_configuration(
        mut self,
        value: CreateInstanceConfigurationDetails,
    ) -> Self {
        self.create_instance_configuration = value;
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
