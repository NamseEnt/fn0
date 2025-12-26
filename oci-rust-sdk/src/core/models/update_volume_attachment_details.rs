use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// details for updating a volume attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVolumeAttachmentDetails {
    /// The iscsi login state of the volume attachment. For a multipath volume attachment, all iscsi sessions need to be all logged-in or logged-out to be in logged-in or logged-out state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iscsi_login_state: Option<UpdateVolumeAttachmentDetailsIscsiLoginState>,
}

impl UpdateVolumeAttachmentDetails {
    /// Create a new UpdateVolumeAttachmentDetails
    pub fn new() -> Self {
        Self {
            iscsi_login_state: None,
        }
    }

    /// Set iscsi_login_state
    pub fn set_iscsi_login_state(
        mut self,
        value: Option<UpdateVolumeAttachmentDetailsIscsiLoginState>,
    ) -> Self {
        self.iscsi_login_state = value;
        self
    }

    /// Set iscsi_login_state (unwraps Option)
    pub fn with_iscsi_login_state(
        mut self,
        value: UpdateVolumeAttachmentDetailsIscsiLoginState,
    ) -> Self {
        self.iscsi_login_state = Some(value);
        self
    }
}

impl Default for UpdateVolumeAttachmentDetails {
    fn default() -> Self {
        Self::new()
    }
}
