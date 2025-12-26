use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Terminates the preemptible instance when it is interrupted for eviction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminatePreemptionAction {
    #[serde(rename = "type")]
    pub r#type: String,

    /// Whether to preserve the boot volume that was used to launch the preemptible instance when the instance is terminated. Defaults to false if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_boot_volume: Option<bool>,
}

/// Required fields for TerminatePreemptionAction
pub struct TerminatePreemptionActionRequired {
    pub r#type: String,
}

impl TerminatePreemptionAction {
    /// Create a new TerminatePreemptionAction with required fields
    pub fn new(required: TerminatePreemptionActionRequired) -> Self {
        Self {
            r#type: required.r#type,

            preserve_boot_volume: None,
        }
    }

    /// Set preserve_boot_volume
    pub fn set_preserve_boot_volume(mut self, value: Option<bool>) -> Self {
        self.preserve_boot_volume = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set preserve_boot_volume (unwraps Option)
    pub fn with_preserve_boot_volume(mut self, value: bool) -> Self {
        self.preserve_boot_volume = Some(value);
        self
    }
}
