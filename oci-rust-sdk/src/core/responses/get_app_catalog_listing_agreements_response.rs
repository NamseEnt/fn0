use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAppCatalogListingAgreementsResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.AppCatalogListingResourceVersionAgreements instance.
    pub app_catalog_listing_resource_version_agreements: AppCatalogListingResourceVersionAgreements,
}


/// Required fields for GetAppCatalogListingAgreementsResponse
pub struct GetAppCatalogListingAgreementsResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.AppCatalogListingResourceVersionAgreements instance.
    pub app_catalog_listing_resource_version_agreements: AppCatalogListingResourceVersionAgreements,
}

impl GetAppCatalogListingAgreementsResponse {
    /// Create a new GetAppCatalogListingAgreementsResponse with required fields
    pub fn new(required: GetAppCatalogListingAgreementsResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            app_catalog_listing_resource_version_agreements: required.app_catalog_listing_resource_version_agreements,
}
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set app_catalog_listing_resource_version_agreements
    pub fn set_app_catalog_listing_resource_version_agreements(mut self, value: AppCatalogListingResourceVersionAgreements) -> Self {
        self.app_catalog_listing_resource_version_agreements = value;
        self
    }
}


