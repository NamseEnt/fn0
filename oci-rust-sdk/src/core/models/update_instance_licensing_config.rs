use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The target license config to be updated on the instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstanceLicensingConfig {
    #[serde(rename = "type")]
    pub r#type: String,

    /// License Type for the OS license. * {@code OCI_PROVIDED} - OCI provided license (e.g. metered $/OCPU-hour). * {@code BRING_YOUR_OWN_LICENSE} - Bring your own license.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_type: Option<UpdateInstanceLicensingConfigLicenseType>,
}

/// Required fields for UpdateInstanceLicensingConfig
pub struct UpdateInstanceLicensingConfigRequired {
    pub r#type: String,
}

impl UpdateInstanceLicensingConfig {
    /// Create a new UpdateInstanceLicensingConfig with required fields
    pub fn new(required: UpdateInstanceLicensingConfigRequired) -> Self {
        Self {
            r#type: required.r#type,

            license_type: None,
        }
    }

    /// Set license_type
    pub fn set_license_type(
        mut self,
        value: Option<UpdateInstanceLicensingConfigLicenseType>,
    ) -> Self {
        self.license_type = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set license_type (unwraps Option)
    pub fn with_license_type(mut self, value: UpdateInstanceLicensingConfigLicenseType) -> Self {
        self.license_type = Some(value);
        self
    }
}
