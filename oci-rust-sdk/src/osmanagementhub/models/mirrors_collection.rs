use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of software source mirrors returned for the {@link #listMirrors(ListMirrorsRequest) listMirrors} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorsCollection {
    /// List of mirrors
    pub items: Vec<MirrorSummary>,
}

/// Required fields for MirrorsCollection
pub struct MirrorsCollectionRequired {
    /// List of mirrors
    pub items: Vec<MirrorSummary>,
}

impl MirrorsCollection {
    /// Create a new MirrorsCollection with required fields
    pub fn new(required: MirrorsCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<MirrorSummary>) -> Self {
        self.items = value;
        self
    }
}
