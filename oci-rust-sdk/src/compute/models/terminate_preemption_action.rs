use serde::{Deserialize, Serialize};

/// Terminates the preemptible instance when it is interrupted for eviction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminatePreemptionAction {
    /// Whether to preserve the boot volume that was used to launch the preemptible instance
    /// when the instance is terminated. Defaults to false if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_boot_volume: Option<bool>,
}

impl TerminatePreemptionAction {
    pub fn new() -> Self {
        Self {
            preserve_boot_volume: None,
        }
    }

    pub fn with_preserve_boot_volume(mut self, preserve: bool) -> Self {
        self.preserve_boot_volume = Some(preserve);
        self
    }
}

impl Default for TerminatePreemptionAction {
    fn default() -> Self {
        Self::new()
    }
}
