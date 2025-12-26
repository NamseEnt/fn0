use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAppCatalogListingAgreementsRequest {
    /// The OCID of the listing.
    pub listing_id: String,

    /// Listing Resource Version.
    pub resource_version: String,
}

/// Required fields for GetAppCatalogListingAgreementsRequest
pub struct GetAppCatalogListingAgreementsRequestRequired {
    /// The OCID of the listing.
    pub listing_id: String,

    /// Listing Resource Version.
    pub resource_version: String,
}

impl GetAppCatalogListingAgreementsRequest {
    /// Create a new GetAppCatalogListingAgreementsRequest with required fields
    pub fn new(required: GetAppCatalogListingAgreementsRequestRequired) -> Self {
        Self {
            listing_id: required.listing_id,

            resource_version: required.resource_version,
        }
    }

    /// Set listing_id
    pub fn set_listing_id(mut self, value: String) -> Self {
        self.listing_id = value;
        self
    }

    /// Set resource_version
    pub fn set_resource_version(mut self, value: String) -> Self {
        self.resource_version = value;
        self
    }
}
