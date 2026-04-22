use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides additional information for a reboot event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebootEventData {
    /// Reboot status for the current event
    pub reboot_status: RebootEventDataRebootStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_details: Option<WorkRequestEventDataAdditionalDetails>,
}

/// Required fields for RebootEventData
pub struct RebootEventDataRequired {
    /// Reboot status for the current event
    pub reboot_status: RebootEventDataRebootStatus,
}

impl RebootEventData {
    /// Create a new RebootEventData with required fields
    pub fn new(required: RebootEventDataRequired) -> Self {
        Self {
            reboot_status: required.reboot_status,

            additional_details: None,
        }
    }

    /// Set reboot_status
    pub fn set_reboot_status(mut self, value: RebootEventDataRebootStatus) -> Self {
        self.reboot_status = value;
        self
    }

    /// Set additional_details
    pub fn set_additional_details(
        mut self,
        value: Option<WorkRequestEventDataAdditionalDetails>,
    ) -> Self {
        self.additional_details = value;
        self
    }

    /// Set additional_details (unwraps Option)
    pub fn with_additional_details(mut self, value: WorkRequestEventDataAdditionalDetails) -> Self {
        self.additional_details = Some(value);
        self
    }
}
