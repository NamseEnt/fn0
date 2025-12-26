use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Agreements for a listing resource version.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppCatalogListingResourceVersionAgreements {
    /// The OCID of the listing associated with these agreements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing_id: Option<String>,

    /// Listing resource version associated with these agreements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing_resource_version: Option<String>,

    /// Oracle TOU link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_terms_of_use_link: Option<String>,

    /// EULA link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eula_link: Option<String>,

    /// Date and time the agreements were retrieved, in [RFC3339](https://tools.ietf.org/html/rfc3339) format. Example: {@code 2018-03-20T12:32:53.532Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_retrieved: Option<DateTime<Utc>>,

    /// A generated signature for this agreement retrieval operation which should be used in the create subscription call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl AppCatalogListingResourceVersionAgreements {
    /// Create a new AppCatalogListingResourceVersionAgreements
    pub fn new() -> Self {
        Self {
            listing_id: None,

            listing_resource_version: None,

            oracle_terms_of_use_link: None,

            eula_link: None,

            time_retrieved: None,

            signature: None,
        }
    }

    /// Set listing_id
    pub fn set_listing_id(mut self, value: Option<String>) -> Self {
        self.listing_id = value;
        self
    }

    /// Set listing_resource_version
    pub fn set_listing_resource_version(mut self, value: Option<String>) -> Self {
        self.listing_resource_version = value;
        self
    }

    /// Set oracle_terms_of_use_link
    pub fn set_oracle_terms_of_use_link(mut self, value: Option<String>) -> Self {
        self.oracle_terms_of_use_link = value;
        self
    }

    /// Set eula_link
    pub fn set_eula_link(mut self, value: Option<String>) -> Self {
        self.eula_link = value;
        self
    }

    /// Set time_retrieved
    pub fn set_time_retrieved(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_retrieved = value;
        self
    }

    /// Set signature
    pub fn set_signature(mut self, value: Option<String>) -> Self {
        self.signature = value;
        self
    }

    /// Set listing_id (unwraps Option)
    pub fn with_listing_id(mut self, value: impl Into<String>) -> Self {
        self.listing_id = Some(value.into());
        self
    }

    /// Set listing_resource_version (unwraps Option)
    pub fn with_listing_resource_version(mut self, value: impl Into<String>) -> Self {
        self.listing_resource_version = Some(value.into());
        self
    }

    /// Set oracle_terms_of_use_link (unwraps Option)
    pub fn with_oracle_terms_of_use_link(mut self, value: impl Into<String>) -> Self {
        self.oracle_terms_of_use_link = Some(value.into());
        self
    }

    /// Set eula_link (unwraps Option)
    pub fn with_eula_link(mut self, value: impl Into<String>) -> Self {
        self.eula_link = Some(value.into());
        self
    }

    /// Set time_retrieved (unwraps Option)
    pub fn with_time_retrieved(mut self, value: DateTime<Utc>) -> Self {
        self.time_retrieved = Some(value);
        self
    }

    /// Set signature (unwraps Option)
    pub fn with_signature(mut self, value: impl Into<String>) -> Self {
        self.signature = Some(value.into());
        self
    }
}

impl Default for AppCatalogListingResourceVersionAgreements {
    fn default() -> Self {
        Self::new()
    }
}
