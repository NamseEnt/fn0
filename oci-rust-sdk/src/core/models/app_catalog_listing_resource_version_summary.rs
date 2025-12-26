use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Listing Resource Version summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppCatalogListingResourceVersionSummary {
    /// The OCID of the listing this resource version belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing_id: Option<String>,

    /// Date and time the listing resource version was published, in [RFC3339](https://tools.ietf.org/html/rfc3339) format. Example: {@code 2018-03-20T12:32:53.532Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_published: Option<DateTime<Utc>>,

    /// OCID of the listing resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing_resource_id: Option<String>,

    /// Resource Version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing_resource_version: Option<String>,
}

impl AppCatalogListingResourceVersionSummary {
    /// Create a new AppCatalogListingResourceVersionSummary
    pub fn new() -> Self {
        Self {
            listing_id: None,

            time_published: None,

            listing_resource_id: None,

            listing_resource_version: None,
        }
    }

    /// Set listing_id
    pub fn set_listing_id(mut self, value: Option<String>) -> Self {
        self.listing_id = value;
        self
    }

    /// Set time_published
    pub fn set_time_published(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_published = value;
        self
    }

    /// Set listing_resource_id
    pub fn set_listing_resource_id(mut self, value: Option<String>) -> Self {
        self.listing_resource_id = value;
        self
    }

    /// Set listing_resource_version
    pub fn set_listing_resource_version(mut self, value: Option<String>) -> Self {
        self.listing_resource_version = value;
        self
    }

    /// Set listing_id (unwraps Option)
    pub fn with_listing_id(mut self, value: impl Into<String>) -> Self {
        self.listing_id = Some(value.into());
        self
    }

    /// Set time_published (unwraps Option)
    pub fn with_time_published(mut self, value: DateTime<Utc>) -> Self {
        self.time_published = Some(value);
        self
    }

    /// Set listing_resource_id (unwraps Option)
    pub fn with_listing_resource_id(mut self, value: impl Into<String>) -> Self {
        self.listing_resource_id = Some(value.into());
        self
    }

    /// Set listing_resource_version (unwraps Option)
    pub fn with_listing_resource_version(mut self, value: impl Into<String>) -> Self {
        self.listing_resource_version = Some(value.into());
        self
    }
}

impl Default for AppCatalogListingResourceVersionSummary {
    fn default() -> Self {
        Self::new()
    }
}
