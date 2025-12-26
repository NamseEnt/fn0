use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Listing details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppCatalogListing {
    /// Listing's contact URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_url: Option<String>,

    /// Description of the listing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The OCID of the listing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing_id: Option<String>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Date and time the listing was published, in [RFC3339](https://tools.ietf.org/html/rfc3339) format. Example: {@code 2018-03-20T12:32:53.532Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_published: Option<DateTime<Utc>>,

    /// Publisher's logo URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_logo_url: Option<String>,

    /// Name of the publisher who published this listing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_name: Option<String>,

    /// Summary of the listing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

impl AppCatalogListing {
    /// Create a new AppCatalogListing
    pub fn new() -> Self {
        Self {
            contact_url: None,

            description: None,

            listing_id: None,

            display_name: None,

            time_published: None,

            publisher_logo_url: None,

            publisher_name: None,

            summary: None,
        }
    }

    /// Set contact_url
    pub fn set_contact_url(mut self, value: Option<String>) -> Self {
        self.contact_url = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set listing_id
    pub fn set_listing_id(mut self, value: Option<String>) -> Self {
        self.listing_id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set time_published
    pub fn set_time_published(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_published = value;
        self
    }

    /// Set publisher_logo_url
    pub fn set_publisher_logo_url(mut self, value: Option<String>) -> Self {
        self.publisher_logo_url = value;
        self
    }

    /// Set publisher_name
    pub fn set_publisher_name(mut self, value: Option<String>) -> Self {
        self.publisher_name = value;
        self
    }

    /// Set summary
    pub fn set_summary(mut self, value: Option<String>) -> Self {
        self.summary = value;
        self
    }

    /// Set contact_url (unwraps Option)
    pub fn with_contact_url(mut self, value: impl Into<String>) -> Self {
        self.contact_url = Some(value.into());
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set listing_id (unwraps Option)
    pub fn with_listing_id(mut self, value: impl Into<String>) -> Self {
        self.listing_id = Some(value.into());
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set time_published (unwraps Option)
    pub fn with_time_published(mut self, value: DateTime<Utc>) -> Self {
        self.time_published = Some(value);
        self
    }

    /// Set publisher_logo_url (unwraps Option)
    pub fn with_publisher_logo_url(mut self, value: impl Into<String>) -> Self {
        self.publisher_logo_url = Some(value.into());
        self
    }

    /// Set publisher_name (unwraps Option)
    pub fn with_publisher_name(mut self, value: impl Into<String>) -> Self {
        self.publisher_name = Some(value.into());
        self
    }

    /// Set summary (unwraps Option)
    pub fn with_summary(mut self, value: impl Into<String>) -> Self {
        self.summary = Some(value.into());
        self
    }
}

impl Default for AppCatalogListing {
    fn default() -> Self {
        Self::new()
    }
}
