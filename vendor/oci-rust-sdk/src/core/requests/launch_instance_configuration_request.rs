use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchInstanceConfigurationRequest {
    /// The OCID of the instance configuration.
    pub instance_configuration_id: String,

    /// Instance configuration Instance Details
    pub instance_configuration: ComputeInstanceOptions,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,
}

/// Required fields for LaunchInstanceConfigurationRequest
pub struct LaunchInstanceConfigurationRequestRequired {
    /// The OCID of the instance configuration.
    pub instance_configuration_id: String,

    /// Instance configuration Instance Details
    pub instance_configuration: ComputeInstanceOptions,
}

impl LaunchInstanceConfigurationRequest {
    /// Create a new LaunchInstanceConfigurationRequest with required fields
    pub fn new(required: LaunchInstanceConfigurationRequestRequired) -> Self {
        Self {
            instance_configuration_id: required.instance_configuration_id,

            instance_configuration: required.instance_configuration,

            opc_retry_token: None,
        }
    }

    /// Set instance_configuration_id
    pub fn set_instance_configuration_id(mut self, value: String) -> Self {
        self.instance_configuration_id = value;
        self
    }

    /// Set instance_configuration
    pub fn set_instance_configuration(mut self, value: ComputeInstanceOptions) -> Self {
        self.instance_configuration = value;
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
