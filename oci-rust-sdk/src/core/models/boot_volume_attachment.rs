use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Represents an attachment between a boot volume and an instance. <p> *Warning:** Oracle recommends that you avoid using any confidential information when you supply string values using the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootVolumeAttachment {
    /// The availability domain of an instance. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the boot volume.
    pub boot_volume_id: String,

    /// The OCID of the compartment.
    pub compartment_id: String,

    /// The OCID of the boot volume attachment.
    pub id: String,

    /// The OCID of the instance the boot volume is attached to.
    pub instance_id: String,

    /// The current state of the boot volume attachment.
    pub lifecycle_state: BootVolumeAttachmentLifecycleState,

    /// The date and time the boot volume was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The date and time the boot volume attachment was updated, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_updated: Option<DateTime<Utc>>,

    /// Whether in-transit encryption for the boot volume's paravirtualized attachment is enabled or not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pv_encryption_in_transit_enabled: Option<bool>,

    /// Refer the top-level definition of encryptionInTransitType. The default value is NONE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_in_transit_type: Option<EncryptionInTransitType>,
}

/// Required fields for BootVolumeAttachment
pub struct BootVolumeAttachmentRequired {
    /// The availability domain of an instance. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the boot volume.
    pub boot_volume_id: String,

    /// The OCID of the compartment.
    pub compartment_id: String,

    /// The OCID of the boot volume attachment.
    pub id: String,

    /// The OCID of the instance the boot volume is attached to.
    pub instance_id: String,

    /// The current state of the boot volume attachment.
    pub lifecycle_state: BootVolumeAttachmentLifecycleState,

    /// The date and time the boot volume was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl BootVolumeAttachment {
    /// Create a new BootVolumeAttachment with required fields
    pub fn new(required: BootVolumeAttachmentRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            boot_volume_id: required.boot_volume_id,

            compartment_id: required.compartment_id,

            id: required.id,

            instance_id: required.instance_id,

            lifecycle_state: required.lifecycle_state,

            time_created: required.time_created,

            display_name: None,

            time_updated: None,

            is_pv_encryption_in_transit_enabled: None,

            encryption_in_transit_type: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set boot_volume_id
    pub fn set_boot_volume_id(mut self, value: String) -> Self {
        self.boot_volume_id = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
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

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: BootVolumeAttachmentLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set time_updated
    pub fn set_time_updated(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_updated = value;
        self
    }

    /// Set is_pv_encryption_in_transit_enabled
    pub fn set_is_pv_encryption_in_transit_enabled(mut self, value: Option<bool>) -> Self {
        self.is_pv_encryption_in_transit_enabled = value;
        self
    }

    /// Set encryption_in_transit_type
    pub fn set_encryption_in_transit_type(
        mut self,
        value: Option<EncryptionInTransitType>,
    ) -> Self {
        self.encryption_in_transit_type = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set time_updated (unwraps Option)
    pub fn with_time_updated(mut self, value: DateTime<Utc>) -> Self {
        self.time_updated = Some(value);
        self
    }

    /// Set is_pv_encryption_in_transit_enabled (unwraps Option)
    pub fn with_is_pv_encryption_in_transit_enabled(mut self, value: bool) -> Self {
        self.is_pv_encryption_in_transit_enabled = Some(value);
        self
    }

    /// Set encryption_in_transit_type (unwraps Option)
    pub fn with_encryption_in_transit_type(mut self, value: EncryptionInTransitType) -> Self {
        self.encryption_in_transit_type = Some(value);
        self
    }
}
