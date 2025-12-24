use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An object that provides information about an update for a Windows instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsUpdate {
    /// Name of the Windows update.
    pub name: String,

    /// Unique identifier for the Windows update. Note that this is not an OCID, but is a unique identifier assigned by Microsoft. Example: '6981d463-cd91-4a26-b7c4-ea4ded9183ed'
    pub update_id: String,

    /// The type of Windows update.
    pub update_type: ClassificationTypes,

    /// Description of the update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// size of the package in bytes Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_in_bytes: Option<i64>,

    /// Indicates whether the update can be installed using the service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installable: Option<WindowsUpdateInstallable>,

    /// List of requirements for installing the update on the managed instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installation_requirements: Option<Vec<InstallationRequirements>>,

    /// Indicates whether a reboot is required to complete the installation of this update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_reboot_required_for_installation: Option<bool>,

    /// List of the Microsoft Knowledge Base Article Ids related to this Windows Update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kb_article_ids: Option<Vec<String>>,
}

/// Required fields for WindowsUpdate
pub struct WindowsUpdateRequired {
    /// Name of the Windows update.
    pub name: String,

    /// Unique identifier for the Windows update. Note that this is not an OCID, but is a unique identifier assigned by Microsoft. Example: '6981d463-cd91-4a26-b7c4-ea4ded9183ed'
    pub update_id: String,

    /// The type of Windows update.
    pub update_type: ClassificationTypes,
}

impl WindowsUpdate {
    /// Create a new WindowsUpdate with required fields
    pub fn new(required: WindowsUpdateRequired) -> Self {
        Self {
            name: required.name,

            update_id: required.update_id,

            update_type: required.update_type,

            description: None,

            size_in_bytes: None,

            installable: None,

            installation_requirements: None,

            is_reboot_required_for_installation: None,

            kb_article_ids: None,
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

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set update_type
    pub fn set_update_type(mut self, value: ClassificationTypes) -> Self {
        self.update_type = value;
        self
    }

    /// Set size_in_bytes
    pub fn set_size_in_bytes(mut self, value: Option<i64>) -> Self {
        self.size_in_bytes = value;
        self
    }

    /// Set installable
    pub fn set_installable(mut self, value: Option<WindowsUpdateInstallable>) -> Self {
        self.installable = value;
        self
    }

    /// Set installation_requirements
    pub fn set_installation_requirements(
        mut self,
        value: Option<Vec<InstallationRequirements>>,
    ) -> Self {
        self.installation_requirements = value;
        self
    }

    /// Set is_reboot_required_for_installation
    pub fn set_is_reboot_required_for_installation(mut self, value: Option<bool>) -> Self {
        self.is_reboot_required_for_installation = value;
        self
    }

    /// Set kb_article_ids
    pub fn set_kb_article_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.kb_article_ids = value;
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set size_in_bytes (unwraps Option)
    pub fn with_size_in_bytes(mut self, value: i64) -> Self {
        self.size_in_bytes = Some(value);
        self
    }

    /// Set installable (unwraps Option)
    pub fn with_installable(mut self, value: WindowsUpdateInstallable) -> Self {
        self.installable = Some(value);
        self
    }

    /// Set installation_requirements (unwraps Option)
    pub fn with_installation_requirements(mut self, value: Vec<InstallationRequirements>) -> Self {
        self.installation_requirements = Some(value);
        self
    }

    /// Set is_reboot_required_for_installation (unwraps Option)
    pub fn with_is_reboot_required_for_installation(mut self, value: bool) -> Self {
        self.is_reboot_required_for_installation = Some(value);
        self
    }

    /// Set kb_article_ids (unwraps Option)
    pub fn with_kb_article_ids(mut self, value: Vec<String>) -> Self {
        self.kb_article_ids = Some(value);
        self
    }
}
