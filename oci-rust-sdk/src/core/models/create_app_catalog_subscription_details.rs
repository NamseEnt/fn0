use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// details for creating a subscription for a listing resource version.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAppCatalogSubscriptionDetails {
    /// The compartmentID for the subscription.
    pub compartment_id: String,

    /// The OCID of the listing.
    pub listing_id: String,

    /// Listing resource version.
    pub listing_resource_version: String,

    /// Oracle TOU link
    pub oracle_terms_of_use_link: String,

    /// Date and time the agreements were retrieved, in [RFC3339](https://tools.ietf.org/html/rfc3339) format. Example: {@code 2018-03-20T12:32:53.532Z}
    pub time_retrieved: DateTime<Utc>,

    /// A generated signature for this listing resource version retrieved the agreements API.
    pub signature: String,

    /// EULA link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eula_link: Option<String>,
}

/// Required fields for CreateAppCatalogSubscriptionDetails
pub struct CreateAppCatalogSubscriptionDetailsRequired {
    /// The compartmentID for the subscription.
    pub compartment_id: String,

    /// The OCID of the listing.
    pub listing_id: String,

    /// Listing resource version.
    pub listing_resource_version: String,

    /// Oracle TOU link
    pub oracle_terms_of_use_link: String,

    /// Date and time the agreements were retrieved, in [RFC3339](https://tools.ietf.org/html/rfc3339) format. Example: {@code 2018-03-20T12:32:53.532Z}
    pub time_retrieved: DateTime<Utc>,

    /// A generated signature for this listing resource version retrieved the agreements API.
    pub signature: String,
}

impl CreateAppCatalogSubscriptionDetails {
    /// Create a new CreateAppCatalogSubscriptionDetails with required fields
    pub fn new(required: CreateAppCatalogSubscriptionDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            listing_id: required.listing_id,

            listing_resource_version: required.listing_resource_version,

            oracle_terms_of_use_link: required.oracle_terms_of_use_link,

            time_retrieved: required.time_retrieved,

            signature: required.signature,

            eula_link: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set listing_id
    pub fn set_listing_id(mut self, value: String) -> Self {
        self.listing_id = value;
        self
    }

    /// Set listing_resource_version
    pub fn set_listing_resource_version(mut self, value: String) -> Self {
        self.listing_resource_version = value;
        self
    }

    /// Set oracle_terms_of_use_link
    pub fn set_oracle_terms_of_use_link(mut self, value: String) -> Self {
        self.oracle_terms_of_use_link = value;
        self
    }

    /// Set eula_link
    pub fn set_eula_link(mut self, value: Option<String>) -> Self {
        self.eula_link = value;
        self
    }

    /// Set time_retrieved
    pub fn set_time_retrieved(mut self, value: DateTime<Utc>) -> Self {
        self.time_retrieved = value;
        self
    }

    /// Set signature
    pub fn set_signature(mut self, value: String) -> Self {
        self.signature = value;
        self
    }

    /// Set eula_link (unwraps Option)
    pub fn with_eula_link(mut self, value: impl Into<String>) -> Self {
        self.eula_link = Some(value.into());
        self
    }
}
