use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An ISCSI volume attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IScsiVolumeAttachment {
    /// The volume's iSCSI IP address. <p> Example: {@code 169.254.0.2}
    pub ipv4: String,

    /// The target volume's iSCSI Qualified Name in the format defined by [RFC 3720](https://tools.ietf.org/html/rfc3720#page-32). <p> Example: {@code iqn.2015-12.us.oracle.com:<CHAP_username>}
    pub iqn: String,

    /// The volume's iSCSI port, usually port 860 or 3260. <p> Example: {@code 3260} Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub port: i64,

    pub attachment_type: String,

    /// The Challenge-Handshake-Authentication-Protocol (CHAP) secret valid for the associated CHAP user name. (Also called the \"CHAP password\".)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chap_secret: Option<String>,

    /// The volume's system-generated Challenge-Handshake-Authentication-Protocol (CHAP) user name. See [RFC 1994](https://tools.ietf.org/html/rfc1994) for more on CHAP. <p> Example: {@code ocid1.volume.oc1.phx.<unique_ID>}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chap_username: Option<String>,

    /// A list of secondary multipath devices
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multipath_devices: Option<Vec<MultipathDevice>>,

    /// Refer the top-level definition of encryptionInTransitType. The default value is NONE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_in_transit_type: Option<EncryptionInTransitType>,

    /// Whether Oracle Cloud Agent is enabled perform the iSCSI login and logout commands after the volume attach or detach operations for non multipath-enabled iSCSI attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_agent_auto_iscsi_login_enabled: Option<bool>,
}

/// Required fields for IScsiVolumeAttachment
pub struct IScsiVolumeAttachmentRequired {
    /// The volume's iSCSI IP address. <p> Example: {@code 169.254.0.2}
    pub ipv4: String,

    /// The target volume's iSCSI Qualified Name in the format defined by [RFC 3720](https://tools.ietf.org/html/rfc3720#page-32). <p> Example: {@code iqn.2015-12.us.oracle.com:<CHAP_username>}
    pub iqn: String,

    /// The volume's iSCSI port, usually port 860 or 3260. <p> Example: {@code 3260} Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub port: i64,

    pub attachment_type: String,
}

impl IScsiVolumeAttachment {
    /// Create a new IScsiVolumeAttachment with required fields
    pub fn new(required: IScsiVolumeAttachmentRequired) -> Self {
        Self {
            ipv4: required.ipv4,

            iqn: required.iqn,

            port: required.port,

            attachment_type: required.attachment_type,

            chap_secret: None,

            chap_username: None,

            multipath_devices: None,

            encryption_in_transit_type: None,

            is_agent_auto_iscsi_login_enabled: None,
        }
    }

    /// Set chap_secret
    pub fn set_chap_secret(mut self, value: Option<String>) -> Self {
        self.chap_secret = value;
        self
    }

    /// Set chap_username
    pub fn set_chap_username(mut self, value: Option<String>) -> Self {
        self.chap_username = value;
        self
    }

    /// Set ipv4
    pub fn set_ipv4(mut self, value: String) -> Self {
        self.ipv4 = value;
        self
    }

    /// Set iqn
    pub fn set_iqn(mut self, value: String) -> Self {
        self.iqn = value;
        self
    }

    /// Set port
    pub fn set_port(mut self, value: i64) -> Self {
        self.port = value;
        self
    }

    /// Set multipath_devices
    pub fn set_multipath_devices(mut self, value: Option<Vec<MultipathDevice>>) -> Self {
        self.multipath_devices = value;
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

    /// Set attachment_type
    pub fn set_attachment_type(mut self, value: String) -> Self {
        self.attachment_type = value;
        self
    }

    /// Set chap_secret (unwraps Option)
    pub fn with_chap_secret(mut self, value: impl Into<String>) -> Self {
        self.chap_secret = Some(value.into());
        self
    }

    /// Set chap_username (unwraps Option)
    pub fn with_chap_username(mut self, value: impl Into<String>) -> Self {
        self.chap_username = Some(value.into());
        self
    }

    /// Set multipath_devices (unwraps Option)
    pub fn with_multipath_devices(mut self, value: Vec<MultipathDevice>) -> Self {
        self.multipath_devices = Some(value);
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
