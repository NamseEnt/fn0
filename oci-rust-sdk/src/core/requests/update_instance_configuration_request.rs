use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstanceConfigurationRequest {
    /// The OCID of the instance configuration.
    pub instance_configuration_id: String,

    /// Updates the freeFormTags, definedTags, and display name of an instance configuration.
    pub update_instance_configuration_details: UpdateInstanceConfigurationDetails,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for UpdateInstanceConfigurationRequest
pub struct UpdateInstanceConfigurationRequestRequired {
    /// The OCID of the instance configuration.
    pub instance_configuration_id: String,

    /// Updates the freeFormTags, definedTags, and display name of an instance configuration.
    pub update_instance_configuration_details: UpdateInstanceConfigurationDetails,
}

impl UpdateInstanceConfigurationRequest {
    /// Create a new UpdateInstanceConfigurationRequest with required fields
    pub fn new(required: UpdateInstanceConfigurationRequestRequired) -> Self {
        Self {
            instance_configuration_id: required.instance_configuration_id,

            update_instance_configuration_details: required.update_instance_configuration_details,

            opc_retry_token: None,

            if_match: None,
}
    }

    /// Set instance_configuration_id
    pub fn set_instance_configuration_id(mut self, value: String) -> Self {
        self.instance_configuration_id = value;
        self
    }

    /// Set update_instance_configuration_details
    pub fn set_update_instance_configuration_details(mut self, value: UpdateInstanceConfigurationDetails) -> Self {
        self.update_instance_configuration_details = value;
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
}


