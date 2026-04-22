use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The object that defines a custom software source. A software source contains a collection of packages. For more information, see [Managing Software Sources](https://docs.oracle.com/iaas/osmh/doc/software-sources.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomSoftwareSource {
    /// List of vendor software sources that are used for the basis of the custom software source.
    pub vendor_software_sources: Vec<Id>,

    pub software_source_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_software_source_filter: Option<CustomSoftwareSourceFilter>,

    /// Indicates whether the service should automatically update the custom software source to use the latest package versions available. The service reviews packages levels once a day.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_automatically_updated: Option<bool>,

    /// Indicates whether the service should automatically resolve package dependencies when including specific packages in the software source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auto_resolve_dependencies: Option<bool>,

    /// Indicates whether the service should create the software source from a list of packages provided by the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_created_from_package_list: Option<bool>,

    /// Indicates whether the software source will include only the latest versions of content from vendor software sources, while accounting for other constraints set in the custom or versioned custom software source (such as a package list or filters). * For a module filter that does not specify a stream, this will include all available streams, and within each stream only the latest version of packages. * For a module filter that does specify a stream, this will include only the latest version of packages for the specified stream. * For a package filter that does not specify a version, this will include only the latest available version of the package. * For a package filter that does specify a version, this will include only the specified version of the package (the isLatestContentOnly attribute is ignored). * For a package list, this will include only the specified version of packages and modules in the list (the isLatestContentOnly attribute is ignored).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_latest_content_only: Option<bool>,

    /// The packages in the software source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packages: Option<Vec<String>>,

    /// Identifies how the custom software source was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_sub_type: Option<SoftwareSourceSubType>,

    /// The date and time the metadata for this software source was last updated (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_metadata_updated: Option<DateTime<Utc>>,
}

/// Required fields for CustomSoftwareSource
pub struct CustomSoftwareSourceRequired {
    /// List of vendor software sources that are used for the basis of the custom software source.
    pub vendor_software_sources: Vec<Id>,

    pub software_source_type: String,
}

impl CustomSoftwareSource {
    /// Create a new CustomSoftwareSource with required fields
    pub fn new(required: CustomSoftwareSourceRequired) -> Self {
        Self {
            vendor_software_sources: required.vendor_software_sources,

            software_source_type: required.software_source_type,

            custom_software_source_filter: None,

            is_automatically_updated: None,

            is_auto_resolve_dependencies: None,

            is_created_from_package_list: None,

            is_latest_content_only: None,

            packages: None,

            software_source_sub_type: None,

            time_metadata_updated: None,
        }
    }

    /// Set vendor_software_sources
    pub fn set_vendor_software_sources(mut self, value: Vec<Id>) -> Self {
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

    /// Set is_created_from_package_list
    pub fn set_is_created_from_package_list(mut self, value: Option<bool>) -> Self {
        self.is_created_from_package_list = value;
        self
    }

    /// Set is_latest_content_only
    pub fn set_is_latest_content_only(mut self, value: Option<bool>) -> Self {
        self.is_latest_content_only = value;
        self
    }

    /// Set packages
    pub fn set_packages(mut self, value: Option<Vec<String>>) -> Self {
        self.packages = value;
        self
    }

    /// Set software_source_sub_type
    pub fn set_software_source_sub_type(mut self, value: Option<SoftwareSourceSubType>) -> Self {
        self.software_source_sub_type = value;
        self
    }

    /// Set time_metadata_updated
    pub fn set_time_metadata_updated(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_metadata_updated = value;
        self
    }

    /// Set software_source_type
    pub fn set_software_source_type(mut self, value: String) -> Self {
        self.software_source_type = value;
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

    /// Set is_created_from_package_list (unwraps Option)
    pub fn with_is_created_from_package_list(mut self, value: bool) -> Self {
        self.is_created_from_package_list = Some(value);
        self
    }

    /// Set is_latest_content_only (unwraps Option)
    pub fn with_is_latest_content_only(mut self, value: bool) -> Self {
        self.is_latest_content_only = Some(value);
        self
    }

    /// Set packages (unwraps Option)
    pub fn with_packages(mut self, value: Vec<String>) -> Self {
        self.packages = Some(value);
        self
    }

    /// Set software_source_sub_type (unwraps Option)
    pub fn with_software_source_sub_type(mut self, value: SoftwareSourceSubType) -> Self {
        self.software_source_sub_type = Some(value);
        self
    }

    /// Set time_metadata_updated (unwraps Option)
    pub fn with_time_metadata_updated(mut self, value: DateTime<Utc>) -> Self {
        self.time_metadata_updated = Some(value);
        self
    }
}
