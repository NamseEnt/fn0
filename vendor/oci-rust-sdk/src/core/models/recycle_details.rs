use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Shows details about the last recycle performed on this host.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecycleDetails {
    /// Preferred recycle level for hosts associated with the reservation config. * {@code SKIP_RECYCLE} - Skips host wipe. * {@code FULL_RECYCLE} - Does not skip host wipe. This is the default behavior.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recycle_level: Option<RecycleDetailsRecycleLevel>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute host group this host was attached to at the time of recycle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_host_group_id: Option<String>,
}

impl RecycleDetails {
    /// Create a new RecycleDetails
    pub fn new() -> Self {
        Self {
            recycle_level: None,

            compute_host_group_id: None,
        }
    }

    /// Set recycle_level
    pub fn set_recycle_level(mut self, value: Option<RecycleDetailsRecycleLevel>) -> Self {
        self.recycle_level = value;
        self
    }

    /// Set compute_host_group_id
    pub fn set_compute_host_group_id(mut self, value: Option<String>) -> Self {
        self.compute_host_group_id = value;
        self
    }

    /// Set recycle_level (unwraps Option)
    pub fn with_recycle_level(mut self, value: RecycleDetailsRecycleLevel) -> Self {
        self.recycle_level = Some(value);
        self
    }

    /// Set compute_host_group_id (unwraps Option)
    pub fn with_compute_host_group_id(mut self, value: impl Into<String>) -> Self {
        self.compute_host_group_id = Some(value.into());
        self
    }
}

impl Default for RecycleDetails {
    fn default() -> Self {
        Self::new()
    }
}
