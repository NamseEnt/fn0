use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of entitlements returned for the {@link #listEntitlements(ListEntitlementsRequest) listEntitlements}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntitlementCollection {
    /// List of entitlements.
    pub items: Vec<EntitlementSummary>,
}

/// Required fields for EntitlementCollection
pub struct EntitlementCollectionRequired {
    /// List of entitlements.
    pub items: Vec<EntitlementSummary>,
}

impl EntitlementCollection {
    /// Create a new EntitlementCollection with required fields
    pub fn new(required: EntitlementCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<EntitlementSummary>) -> Self {
        self.items = value;
        self
    }
}
