use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A base object for all types of attachments between a storage volume and an instance. For specific details about iSCSI attachments, see {@link IScsiVolumeAttachment}. <p> For general information about volume attachments, see [Overview of Block Volume Storage](https://docs.oracle.com/iaas/Content/Block/Concepts/overview.htm). <p> *Warning:** Oracle recommends that you avoid using any confidential information when you supply string values using the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeAttachment {
    /// The availability domain of an instance. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment.
    pub compartment_id: String,

    /// The OCID of the volume attachment.
    pub id: String,

    /// The OCID of the instance the volume is attached to.
    pub instance_id: String,

    /// The current state of the volume attachment.
    pub lifecycle_state: VolumeAttachmentLifecycleState,

    /// The date and time the volume was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The OCID of the volume.
    pub volume_id: String,

    pub attachment_type: String,

    /// The device name.
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

    /// Whether in-transit encryption for the data volume's paravirtualized attachment is enabled or not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pv_encryption_in_transit_enabled: Option<bool>,

    /// Whether the Iscsi or Paravirtualized attachment is multipath or not, it is not applicable to NVMe attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_multipath: Option<bool>,

    /// The iscsi login state of the volume attachment. For a Iscsi volume attachment, all iscsi sessions need to be all logged-in or logged-out to be in logged-in or logged-out state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iscsi_login_state: Option<VolumeAttachmentIscsiLoginState>,

    /// Flag indicating if this volume was created for the customer as part of a simplified launch. Used to determine whether the volume requires deletion on instance termination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_volume_created_during_launch: Option<bool>,
}

/// Required fields for VolumeAttachment
pub struct VolumeAttachmentRequired {
    /// The availability domain of an instance. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment.
    pub compartment_id: String,

    /// The OCID of the volume attachment.
    pub id: String,

    /// The OCID of the instance the volume is attached to.
    pub instance_id: String,

    /// The current state of the volume attachment.
    pub lifecycle_state: VolumeAttachmentLifecycleState,

    /// The date and time the volume was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The OCID of the volume.
    pub volume_id: String,

    pub attachment_type: String,
}

impl VolumeAttachment {
    /// Create a new VolumeAttachment with required fields
    pub fn new(required: VolumeAttachmentRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            id: required.id,

            instance_id: required.instance_id,

            lifecycle_state: required.lifecycle_state,

            time_created: required.time_created,

            volume_id: required.volume_id,

            attachment_type: required.attachment_type,

            device: None,

            display_name: None,

            is_read_only: None,

            is_shareable: None,

            is_pv_encryption_in_transit_enabled: None,

            is_multipath: None,

            iscsi_login_state: None,

            is_volume_created_during_launch: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
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

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
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

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: VolumeAttachmentLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set volume_id
    pub fn set_volume_id(mut self, value: String) -> Self {
        self.volume_id = value;
        self
    }

    /// Set is_pv_encryption_in_transit_enabled
    pub fn set_is_pv_encryption_in_transit_enabled(mut self, value: Option<bool>) -> Self {
        self.is_pv_encryption_in_transit_enabled = value;
        self
    }

    /// Set is_multipath
    pub fn set_is_multipath(mut self, value: Option<bool>) -> Self {
        self.is_multipath = value;
        self
    }

    /// Set iscsi_login_state
    pub fn set_iscsi_login_state(mut self, value: Option<VolumeAttachmentIscsiLoginState>) -> Self {
        self.iscsi_login_state = value;
        self
    }

    /// Set is_volume_created_during_launch
    pub fn set_is_volume_created_during_launch(mut self, value: Option<bool>) -> Self {
        self.is_volume_created_during_launch = value;
        self
    }

    /// Set attachment_type
    pub fn set_attachment_type(mut self, value: String) -> Self {
        self.attachment_type = value;
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

    /// Set is_pv_encryption_in_transit_enabled (unwraps Option)
    pub fn with_is_pv_encryption_in_transit_enabled(mut self, value: bool) -> Self {
        self.is_pv_encryption_in_transit_enabled = Some(value);
        self
    }

    /// Set is_multipath (unwraps Option)
    pub fn with_is_multipath(mut self, value: bool) -> Self {
        self.is_multipath = Some(value);
        self
    }

    /// Set iscsi_login_state (unwraps Option)
    pub fn with_iscsi_login_state(mut self, value: VolumeAttachmentIscsiLoginState) -> Self {
        self.iscsi_login_state = Some(value);
        self
    }

    /// Set is_volume_created_during_launch (unwraps Option)
    pub fn with_is_volume_created_during_launch(mut self, value: bool) -> Self {
        self.is_volume_created_during_launch = Some(value);
        self
    }
}
