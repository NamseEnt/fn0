use crate::compute::models::{Instance, InstanceConfigurationInstanceDetails};

/// Request for launching an instance from an instance configuration.
#[derive(Debug, Clone)]
pub struct LaunchInstanceConfigurationRequest {
    /// The OCID of the instance configuration.
    pub instance_configuration_id: String,

    /// Instance configuration instance details (either Compute or InstanceOptions).
    pub instance_configuration: InstanceConfigurationInstanceDetails,

    /// A token that uniquely identifies a request so it can be retried in case of
    /// a timeout or server error without risk of executing that same action again.
    /// Retry tokens expire after 24 hours.
    pub opc_retry_token: Option<String>,
}

/// Required fields for LaunchInstanceConfigurationRequest.
pub struct LaunchInstanceConfigurationRequestRequiredFields {
    pub instance_configuration_id: String,
    pub instance_configuration: InstanceConfigurationInstanceDetails,
}

impl LaunchInstanceConfigurationRequest {
    /// Creates a new request builder with required fields.
    pub fn builder(
        required: LaunchInstanceConfigurationRequestRequiredFields,
    ) -> LaunchInstanceConfigurationRequestBuilder {
        LaunchInstanceConfigurationRequestBuilder {
            instance_configuration_id: required.instance_configuration_id,
            instance_configuration: required.instance_configuration,
            opc_retry_token: None,
        }
    }
}

/// Builder for LaunchInstanceConfigurationRequest.
pub struct LaunchInstanceConfigurationRequestBuilder {
    instance_configuration_id: String,
    instance_configuration: InstanceConfigurationInstanceDetails,
    opc_retry_token: Option<String>,
}

impl LaunchInstanceConfigurationRequestBuilder {
    /// Sets the retry token for idempotent requests.
    pub fn opc_retry_token(mut self, token: impl Into<String>) -> Self {
        self.opc_retry_token = Some(token.into());
        self
    }

    /// Builds the final request.
    pub fn build(self) -> LaunchInstanceConfigurationRequest {
        LaunchInstanceConfigurationRequest {
            instance_configuration_id: self.instance_configuration_id,
            instance_configuration: self.instance_configuration,
            opc_retry_token: self.opc_retry_token,
        }
    }
}

/// Response from launching an instance configuration.
#[derive(Debug, Clone)]
pub struct LaunchInstanceConfigurationResponse {
    /// The launched instance.
    pub instance: Instance,

    /// For optimistic concurrency control.
    pub etag: Option<String>,

    /// Unique Oracle-assigned identifier for the request.
    pub opc_request_id: Option<String>,

    /// The OCID of the work request. Use GetWorkRequest with this ID
    /// to track the status of the request.
    pub opc_work_request_id: Option<String>,
}
