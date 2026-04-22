use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachIScsiVolumeDetails {
    #[serde(rename = "type")]
    pub r#type: String,

    /// Whether to use CHAP authentication for the volume attachment. Defaults to false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_chap: Option<bool>,

    /// Refer the top-level definition of encryptionInTransitType. The default value is NONE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_in_transit_type: Option<EncryptionInTransitType>,

    /// Whether to enable Oracle Cloud Agent to perform the iSCSI login and logout commands after the volume attach or detach operations for non multipath-enabled iSCSI attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_agent_auto_iscsi_login_enabled: Option<bool>,
}

/// Required fields for AttachIScsiVolumeDetails
pub struct AttachIScsiVolumeDetailsRequired {
    pub r#type: String,
}

impl AttachIScsiVolumeDetails {
    /// Create a new AttachIScsiVolumeDetails with required fields
    pub fn new(required: AttachIScsiVolumeDetailsRequired) -> Self {
        Self {
            r#type: required.r#type,

            use_chap: None,

            encryption_in_transit_type: None,

            is_agent_auto_iscsi_login_enabled: None,
        }
    }

    /// Set use_chap
    pub fn set_use_chap(mut self, value: Option<bool>) -> Self {
        self.use_chap = value;
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

    /// Set is_agent_auto_iscsi_login_enabled
    pub fn set_is_agent_auto_iscsi_login_enabled(mut self, value: Option<bool>) -> Self {
        self.is_agent_auto_iscsi_login_enabled = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set use_chap (unwraps Option)
    pub fn with_use_chap(mut self, value: bool) -> Self {
        self.use_chap = Some(value);
        self
    }

    /// Set encryption_in_transit_type (unwraps Option)
    pub fn with_encryption_in_transit_type(mut self, value: EncryptionInTransitType) -> Self {
        self.encryption_in_transit_type = Some(value);
        self
    }

    /// Set is_agent_auto_iscsi_login_enabled (unwraps Option)
    pub fn with_is_agent_auto_iscsi_login_enabled(mut self, value: bool) -> Self {
        self.is_agent_auto_iscsi_login_enabled = Some(value);
        self
    }
}
