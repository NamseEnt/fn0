use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The redundancy status of the DRG. For more information, see [Redundancy Remedies](https://docs.oracle.com/iaas/Content/Network/Troubleshoot/drgredundancy.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrgRedundancyStatus {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The redundancy status of the DRG.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<DrgRedundancyStatusStatus>,
}

impl DrgRedundancyStatus {
    /// Create a new DrgRedundancyStatus
    pub fn new() -> Self {
        Self {
            id: None,

            status: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: Option<String>) -> Self {
        self.id = value;
        self
    }

    /// Set status
    pub fn set_status(mut self, value: Option<DrgRedundancyStatusStatus>) -> Self {
        self.status = value;
        self
    }

    /// Set id (unwraps Option)
    pub fn with_id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    /// Set status (unwraps Option)
    pub fn with_status(mut self, value: DrgRedundancyStatusStatus) -> Self {
        self.status = Some(value);
        self
    }
}

impl Default for DrgRedundancyStatus {
    fn default() -> Self {
        Self::new()
    }
}
