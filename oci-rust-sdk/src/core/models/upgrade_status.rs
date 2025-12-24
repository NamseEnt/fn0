use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The upgrade status of a DRG.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeStatus {
    /// The {@code drgId} of the upgraded DRG.
    pub drg_id: String,

    /// The current upgrade status of the DRG attachment.
    pub status: UpgradeStatusStatus,

    /// The number of upgraded connections.
    pub upgraded_connections: String,
}

/// Required fields for UpgradeStatus
pub struct UpgradeStatusRequired {
    /// The {@code drgId} of the upgraded DRG.
    pub drg_id: String,

    /// The current upgrade status of the DRG attachment.
    pub status: UpgradeStatusStatus,

    /// The number of upgraded connections.
    pub upgraded_connections: String,
}

impl UpgradeStatus {
    /// Create a new UpgradeStatus with required fields
    pub fn new(required: UpgradeStatusRequired) -> Self {
        Self {
            drg_id: required.drg_id,

            status: required.status,

            upgraded_connections: required.upgraded_connections,
        }
    }

    /// Set drg_id
    pub fn set_drg_id(mut self, value: String) -> Self {
        self.drg_id = value;
        self
    }

    /// Set status
    pub fn set_status(mut self, value: UpgradeStatusStatus) -> Self {
        self.status = value;
        self
    }

    /// Set upgraded_connections
    pub fn set_upgraded_connections(mut self, value: String) -> Self {
        self.upgraded_connections = value;
        self
    }
}
