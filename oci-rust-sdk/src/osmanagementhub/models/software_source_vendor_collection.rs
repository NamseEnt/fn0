use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Th set of software source vendors returned for the {@link #listSoftwareSourceVendors(ListSoftwareSourceVendorsRequest) listSoftwareSourceVendors} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareSourceVendorCollection {
    /// List of software source vendors.
    pub items: Vec<SoftwareSourceVendorSummary>,
}

/// Required fields for SoftwareSourceVendorCollection
pub struct SoftwareSourceVendorCollectionRequired {
    /// List of software source vendors.
    pub items: Vec<SoftwareSourceVendorSummary>,
}

impl SoftwareSourceVendorCollection {
    /// Create a new SoftwareSourceVendorCollection with required fields
    pub fn new(required: SoftwareSourceVendorCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<SoftwareSourceVendorSummary>) -> Self {
        self.items = value;
        self
    }
}
