use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A summary of a listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppCatalogListingSummary {
    /// the region free ocid of the listing resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing_id: Option<String>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The short summary for the listing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// The name of the publisher who published this listing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_name: Option<String>,
}

impl AppCatalogListingSummary {
    /// Create a new AppCatalogListingSummary
    pub fn new() -> Self {
        Self {
            listing_id: None,

            display_name: None,

            summary: None,

            publisher_name: None,
        }
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

    /// Set summary
    pub fn set_summary(mut self, value: Option<String>) -> Self {
        self.summary = value;
        self
    }

    /// Set publisher_name
    pub fn set_publisher_name(mut self, value: Option<String>) -> Self {
        self.publisher_name = value;
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

    /// Set summary (unwraps Option)
    pub fn with_summary(mut self, value: impl Into<String>) -> Self {
        self.summary = Some(value.into());
        self
    }

    /// Set publisher_name (unwraps Option)
    pub fn with_publisher_name(mut self, value: impl Into<String>) -> Self {
        self.publisher_name = Some(value.into());
        self
    }
}

impl Default for AppCatalogListingSummary {
    fn default() -> Self {
        Self::new()
    }
}
