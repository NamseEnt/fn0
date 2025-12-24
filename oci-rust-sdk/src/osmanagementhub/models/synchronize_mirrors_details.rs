use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Detailed information about software source mirrors to be synced.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SynchronizeMirrorsDetails {
    /// List of software source [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) to synchronize.
    pub software_source_list: Vec<String>,
}

/// Required fields for SynchronizeMirrorsDetails
pub struct SynchronizeMirrorsDetailsRequired {
    /// List of software source [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) to synchronize.
    pub software_source_list: Vec<String>,
}

impl SynchronizeMirrorsDetails {
    /// Create a new SynchronizeMirrorsDetails with required fields
    pub fn new(required: SynchronizeMirrorsDetailsRequired) -> Self {
        Self {
            software_source_list: required.software_source_list,
        }
    }

    /// Set software_source_list
    pub fn set_software_source_list(mut self, value: Vec<String>) -> Self {
        self.software_source_list = value;
        self
    }
}
