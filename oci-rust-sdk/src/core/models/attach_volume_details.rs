use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachVolumeDetails {
    /// The OCID of the instance. For AttachVolume operation, this is a required field for the request, see {@link #attachVolume(AttachVolumeRequest) attachVolume}.
    pub instance_id: String,

    /// The OCID of the volume. If CreateVolumeDetails is specified, this field must be omitted from the request.
    pub volume_id: String,

    #[serde(rename = "type")]
    pub r#type: String,

    /// The device name. To retrieve a list of devices for a given instance, see {@link #listInstanceDevices(ListInstanceDevicesRequest) listInstanceDevices}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Whether the attachment was created in read-only mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_read_only: Option<bool>,

    /// Whether the attachment should be created in shareable mode. If an attachment is created in shareable mode, then other instances can attach the same volume, provided that they also create their attachments in shareable mode. Only certain volume types can be attached in shareable mode. Defaults to false if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_shareable: Option<bool>,
}

/// Required fields for AttachVolumeDetails
pub struct AttachVolumeDetailsRequired {
    /// The OCID of the instance. For AttachVolume operation, this is a required field for the request, see {@link #attachVolume(AttachVolumeRequest) attachVolume}.
    pub instance_id: String,

    /// The OCID of the volume. If CreateVolumeDetails is specified, this field must be omitted from the request.
    pub volume_id: String,

    pub r#type: String,
}

impl AttachVolumeDetails {
    /// Create a new AttachVolumeDetails with required fields
    pub fn new(required: AttachVolumeDetailsRequired) -> Self {
        Self {
            instance_id: required.instance_id,

            volume_id: required.volume_id,

            r#type: required.r#type,

            device: None,

            display_name: None,

            is_read_only: None,

            is_shareable: None,
        }
    }

    /// Set device
    pub fn set_device(mut self, value: Option<String>) -> Self {
        self.device = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set instance_id
    pub fn set_instance_id(mut self, value: String) -> Self {
        self.instance_id = value;
        self
    }

    /// Set is_read_only
    pub fn set_is_read_only(mut self, value: Option<bool>) -> Self {
        self.is_read_only = value;
        self
    }

    /// Set is_shareable
    pub fn set_is_shareable(mut self, value: Option<bool>) -> Self {
        self.is_shareable = value;
        self
    }

    /// Set volume_id
    pub fn set_volume_id(mut self, value: String) -> Self {
        self.volume_id = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set device (unwraps Option)
    pub fn with_device(mut self, value: impl Into<String>) -> Self {
        self.device = Some(value.into());
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set is_read_only (unwraps Option)
    pub fn with_is_read_only(mut self, value: bool) -> Self {
        self.is_read_only = Some(value);
        self
    }

    /// Set is_shareable (unwraps Option)
    pub fn with_is_shareable(mut self, value: bool) -> Self {
        self.is_shareable = Some(value);
        self
    }
}
