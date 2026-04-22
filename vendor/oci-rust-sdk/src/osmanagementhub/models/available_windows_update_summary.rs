use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An object that defines an available update for a Windows instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableWindowsUpdateSummary {
    /// Name of the Windows update.
    pub name: String,

    /// Unique identifier for the Windows update. Note that this is not an OCID, but is a unique identifier assigned by Microsoft. Example: '6981d463-cd91-4a26-b7c4-ea4ded9183ed'
    pub update_id: String,

    /// The type of Windows update.
    pub update_type: ClassificationTypes,

    /// Indicates whether the update can be installed using the service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installable: Option<String>,

    /// Indicates whether a reboot is required to complete the installation of this update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_reboot_required_for_installation: Option<bool>,
}

/// Required fields for AvailableWindowsUpdateSummary
pub struct AvailableWindowsUpdateSummaryRequired {
    /// Name of the Windows update.
    pub name: String,

    /// Unique identifier for the Windows update. Note that this is not an OCID, but is a unique identifier assigned by Microsoft. Example: '6981d463-cd91-4a26-b7c4-ea4ded9183ed'
    pub update_id: String,

    /// The type of Windows update.
    pub update_type: ClassificationTypes,
}

impl AvailableWindowsUpdateSummary {
    /// Create a new AvailableWindowsUpdateSummary with required fields
    pub fn new(required: AvailableWindowsUpdateSummaryRequired) -> Self {
        Self {
            name: required.name,

            update_id: required.update_id,

            update_type: required.update_type,

            installable: None,

            is_reboot_required_for_installation: None,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set update_id
    pub fn set_update_id(mut self, value: String) -> Self {
        self.update_id = value;
        self
    }

    /// Set update_type
    pub fn set_update_type(mut self, value: ClassificationTypes) -> Self {
        self.update_type = value;
        self
    }

    /// Set installable
    pub fn set_installable(mut self, value: Option<String>) -> Self {
        self.installable = value;
        self
    }

    /// Set is_reboot_required_for_installation
    pub fn set_is_reboot_required_for_installation(mut self, value: Option<bool>) -> Self {
        self.is_reboot_required_for_installation = value;
        self
    }

    /// Set installable (unwraps Option)
    pub fn with_installable(mut self, value: impl Into<String>) -> Self {
        self.installable = Some(value.into());
        self
    }

    /// Set is_reboot_required_for_installation (unwraps Option)
    pub fn with_is_reboot_required_for_installation(mut self, value: bool) -> Self {
        self.is_reboot_required_for_installation = Some(value);
        self
    }
}
