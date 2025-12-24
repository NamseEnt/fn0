use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// a subscription for a listing resource version.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppCatalogSubscription {
    /// Name of the publisher who published this listing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_name: Option<String>,

    /// The ocid of the listing resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing_id: Option<String>,

    /// Listing resource version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing_resource_version: Option<String>,

    /// Listing resource id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing_resource_id: Option<String>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The short summary to the listing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// The compartmentID of the subscription.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// Date and time at which the subscription was created, in [RFC3339](https://tools.ietf.org/html/rfc3339) format. Example: {@code 2018-03-20T12:32:53.532Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,
}

impl AppCatalogSubscription {
    /// Create a new AppCatalogSubscription
    pub fn new() -> Self {
        Self {
            publisher_name: None,

            listing_id: None,

            listing_resource_version: None,

            listing_resource_id: None,

            display_name: None,

            summary: None,

            compartment_id: None,

            time_created: None,
        }
    }

    /// Set publisher_name
    pub fn set_publisher_name(mut self, value: Option<String>) -> Self {
        self.publisher_name = value;
        self
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

    /// Set listing_resource_id
    pub fn set_listing_resource_id(mut self, value: Option<String>) -> Self {
        self.listing_resource_id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set summary
    pub fn set_summary(mut self, value: Option<String>) -> Self {
        self.summary = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set publisher_name (unwraps Option)
    pub fn with_publisher_name(mut self, value: impl Into<String>) -> Self {
        self.publisher_name = Some(value.into());
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

    /// Set listing_resource_id (unwraps Option)
    pub fn with_listing_resource_id(mut self, value: impl Into<String>) -> Self {
        self.listing_resource_id = Some(value.into());
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set summary (unwraps Option)
    pub fn with_summary(mut self, value: impl Into<String>) -> Self {
        self.summary = Some(value.into());
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }
}

impl Default for AppCatalogSubscription {
    fn default() -> Self {
        Self::new()
    }
}
