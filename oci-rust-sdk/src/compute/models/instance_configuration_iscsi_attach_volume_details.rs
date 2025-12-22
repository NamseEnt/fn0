use serde::{Deserialize, Serialize};

/// ISCSI attach volume details for instance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationIscsiAttachVolumeDetails {
    /// A user-friendly name. Does not have to be unique, and it's changeable.
    /// Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Whether the attachment should be created in read-only mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_read_only: Option<bool>,

    /// The device name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,

    /// Whether the attachment should be created in shareable mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_shareable: Option<bool>,

    /// Whether to use CHAP authentication for the volume attachment. Defaults to false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_chap: Option<bool>,
}

impl InstanceConfigurationIscsiAttachVolumeDetails {
    pub fn new() -> Self {
        Self {
            display_name: None,
            is_read_only: None,
            device: None,
            is_shareable: None,
            use_chap: None,
        }
    }

    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    pub fn with_is_read_only(mut self, read_only: bool) -> Self {
        self.is_read_only = Some(read_only);
        self
    }

    pub fn with_device(mut self, device: impl Into<String>) -> Self {
        self.device = Some(device.into());
        self
    }

    pub fn with_is_shareable(mut self, shareable: bool) -> Self {
        self.is_shareable = Some(shareable);
        self
    }

    pub fn with_use_chap(mut self, use_chap: bool) -> Self {
        self.use_chap = Some(use_chap);
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, display_name: Option<String>) -> Self {
        self.display_name = display_name;
        self
    }

    /// Set is_read_only
    pub fn set_is_read_only(mut self, is_read_only: Option<bool>) -> Self {
        self.is_read_only = is_read_only;
        self
    }

    /// Set device
    pub fn set_device(mut self, device: Option<String>) -> Self {
        self.device = device;
        self
    }

    /// Set is_shareable
    pub fn set_is_shareable(mut self, is_shareable: Option<bool>) -> Self {
        self.is_shareable = is_shareable;
        self
    }

    /// Set use_chap
    pub fn set_use_chap(mut self, use_chap: Option<bool>) -> Self {
        self.use_chap = use_chap;
        self
    }
}

impl Default for InstanceConfigurationIscsiAttachVolumeDetails {
    fn default() -> Self {
        Self::new()
    }
}
