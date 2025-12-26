use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAppCatalogSubscriptionRequest {
    /// The OCID of the listing.
    pub listing_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,

    /// Listing Resource Version.
    pub resource_version: String,
}


/// Required fields for DeleteAppCatalogSubscriptionRequest
pub struct DeleteAppCatalogSubscriptionRequestRequired {
    /// The OCID of the listing.
    pub listing_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,

    /// Listing Resource Version.
    pub resource_version: String,
}

impl DeleteAppCatalogSubscriptionRequest {
    /// Create a new DeleteAppCatalogSubscriptionRequest with required fields
    pub fn new(required: DeleteAppCatalogSubscriptionRequestRequired) -> Self {
        Self {
            listing_id: required.listing_id,

            compartment_id: required.compartment_id,

            resource_version: required.resource_version,
}
    }

    /// Set listing_id
    pub fn set_listing_id(mut self, value: String) -> Self {
        self.listing_id = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set resource_version
    pub fn set_resource_version(mut self, value: String) -> Self {
        self.resource_version = value;
        self
    }
}


