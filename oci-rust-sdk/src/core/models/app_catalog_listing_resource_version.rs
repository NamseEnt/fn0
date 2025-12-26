use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Listing Resource Version
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppCatalogListingResourceVersion {
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

    /// List of regions that this listing resource version is available. <p> For information about regions, see [Regions and Availability Domains](https://docs.oracle.com/iaas/Content/General/Concepts/regions.htm). <p> Example: {@code [\"us-ashburn-1\", \"us-phoenix-1\"]}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_regions: Option<Vec<String>>,

    /// Array of shapes compatible with this resource. <p> You can enumerate all available shapes by calling {@link #listShapes(ListShapesRequest) listShapes}. <p> Example: {@code [\"VM.Standard1.1\", \"VM.Standard1.2\"]}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatible_shapes: Option<Vec<String>>,

    /// List of accessible ports for instances launched with this listing resource version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessible_ports: Option<Vec<i64>>,

    /// Allowed actions for the listing resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_actions: Option<Vec<AppCatalogListingResourceVersionAllowedActions>>,
}

impl AppCatalogListingResourceVersion {
    /// Create a new AppCatalogListingResourceVersion
    pub fn new() -> Self {
        Self {
            listing_id: None,

            time_published: None,

            listing_resource_id: None,

            listing_resource_version: None,

            available_regions: None,

            compatible_shapes: None,

            accessible_ports: None,

            allowed_actions: None,
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

    /// Set available_regions
    pub fn set_available_regions(mut self, value: Option<Vec<String>>) -> Self {
        self.available_regions = value;
        self
    }

    /// Set compatible_shapes
    pub fn set_compatible_shapes(mut self, value: Option<Vec<String>>) -> Self {
        self.compatible_shapes = value;
        self
    }

    /// Set accessible_ports
    pub fn set_accessible_ports(mut self, value: Option<Vec<i64>>) -> Self {
        self.accessible_ports = value;
        self
    }

    /// Set allowed_actions
    pub fn set_allowed_actions(
        mut self,
        value: Option<Vec<AppCatalogListingResourceVersionAllowedActions>>,
    ) -> Self {
        self.allowed_actions = value;
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

    /// Set available_regions (unwraps Option)
    pub fn with_available_regions(mut self, value: Vec<String>) -> Self {
        self.available_regions = Some(value);
        self
    }

    /// Set compatible_shapes (unwraps Option)
    pub fn with_compatible_shapes(mut self, value: Vec<String>) -> Self {
        self.compatible_shapes = Some(value);
        self
    }

    /// Set accessible_ports (unwraps Option)
    pub fn with_accessible_ports(mut self, value: Vec<i64>) -> Self {
        self.accessible_ports = Some(value);
        self
    }

    /// Set allowed_actions (unwraps Option)
    pub fn with_allowed_actions(
        mut self,
        value: Vec<AppCatalogListingResourceVersionAllowedActions>,
    ) -> Self {
        self.allowed_actions = Some(value);
        self
    }
}

impl Default for AppCatalogListingResourceVersion {
    fn default() -> Self {
        Self::new()
    }
}
