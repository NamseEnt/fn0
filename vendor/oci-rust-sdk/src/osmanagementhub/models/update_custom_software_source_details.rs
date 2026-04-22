use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to update a custom software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCustomSoftwareSourceDetails {
    pub software_source_type: String,

    /// List of vendor software sources that are used for the basis of the custom software source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_software_sources: Option<Vec<Id>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_software_source_filter: Option<CustomSoftwareSourceFilter>,

    /// Indicates whether the service should automatically update the custom software source to use the latest package versions available. The service reviews packages levels once a day.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_automatically_updated: Option<bool>,

    /// Indicates whether the service should automatically resolve package dependencies when including specific packages in the software source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auto_resolve_dependencies: Option<bool>,

    /// Indicates whether the software source will include only the latest versions of content from vendor software sources, while accounting for other constraints set in the custom or versioned custom software source (such as a package list or filters). * For a module filter that does not specify a stream, this will include all available streams, and within each stream only the latest version of packages. * For a module filter that does specify a stream, this will include only the latest version of packages for the specified stream. * For a package filter that does not specify a version, this will include only the latest available version of the package. * For a package filter that does specify a version, this will include only the specified version of the package (the isLatestContentOnly attribute is ignored). * For a package list, this will include only the specified version of packages and modules in the list (the isLatestContentOnly attribute is ignored).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_latest_content_only: Option<bool>,
}

/// Required fields for UpdateCustomSoftwareSourceDetails
pub struct UpdateCustomSoftwareSourceDetailsRequired {
    pub software_source_type: String,
}

impl UpdateCustomSoftwareSourceDetails {
    /// Create a new UpdateCustomSoftwareSourceDetails with required fields
    pub fn new(required: UpdateCustomSoftwareSourceDetailsRequired) -> Self {
        Self {
            software_source_type: required.software_source_type,

            vendor_software_sources: None,

            custom_software_source_filter: None,

            is_automatically_updated: None,

            is_auto_resolve_dependencies: None,

            is_latest_content_only: None,
        }
    }

    /// Set vendor_software_sources
    pub fn set_vendor_software_sources(mut self, value: Option<Vec<Id>>) -> Self {
        self.vendor_software_sources = value;
        self
    }

    /// Set custom_software_source_filter
    pub fn set_custom_software_source_filter(
        mut self,
        value: Option<CustomSoftwareSourceFilter>,
    ) -> Self {
        self.custom_software_source_filter = value;
        self
    }

    /// Set is_automatically_updated
    pub fn set_is_automatically_updated(mut self, value: Option<bool>) -> Self {
        self.is_automatically_updated = value;
        self
    }

    /// Set is_auto_resolve_dependencies
    pub fn set_is_auto_resolve_dependencies(mut self, value: Option<bool>) -> Self {
        self.is_auto_resolve_dependencies = value;
        self
    }

    /// Set is_latest_content_only
    pub fn set_is_latest_content_only(mut self, value: Option<bool>) -> Self {
        self.is_latest_content_only = value;
        self
    }

    /// Set software_source_type
    pub fn set_software_source_type(mut self, value: String) -> Self {
        self.software_source_type = value;
        self
    }

    /// Set vendor_software_sources (unwraps Option)
    pub fn with_vendor_software_sources(mut self, value: Vec<Id>) -> Self {
        self.vendor_software_sources = Some(value);
        self
    }

    /// Set custom_software_source_filter (unwraps Option)
    pub fn with_custom_software_source_filter(mut self, value: CustomSoftwareSourceFilter) -> Self {
        self.custom_software_source_filter = Some(value);
        self
    }

    /// Set is_automatically_updated (unwraps Option)
    pub fn with_is_automatically_updated(mut self, value: bool) -> Self {
        self.is_automatically_updated = Some(value);
        self
    }

    /// Set is_auto_resolve_dependencies (unwraps Option)
    pub fn with_is_auto_resolve_dependencies(mut self, value: bool) -> Self {
        self.is_auto_resolve_dependencies = Some(value);
        self
    }

    /// Set is_latest_content_only (unwraps Option)
    pub fn with_is_latest_content_only(mut self, value: bool) -> Self {
        self.is_latest_content_only = Some(value);
        self
    }
}
