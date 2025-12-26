use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAppCatalogListingRequest {
    /// The OCID of the listing.
    pub listing_id: String,
}


/// Required fields for GetAppCatalogListingRequest
pub struct GetAppCatalogListingRequestRequired {
    /// The OCID of the listing.
    pub listing_id: String,
}

impl GetAppCatalogListingRequest {
    /// Create a new GetAppCatalogListingRequest with required fields
    pub fn new(required: GetAppCatalogListingRequestRequired) -> Self {
        Self {
            listing_id: required.listing_id,
}
    }

    /// Set listing_id
    pub fn set_listing_id(mut self, value: String) -> Self {
        self.listing_id = value;
        self
    }
}


