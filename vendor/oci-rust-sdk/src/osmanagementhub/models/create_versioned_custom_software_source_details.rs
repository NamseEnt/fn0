use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to create a versioned custom software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVersionedCustomSoftwareSourceDetails {
    /// List of vendor software sources.
    pub vendor_software_sources: Vec<Id>,

    /// The version to assign to this custom software source.
    pub software_source_version: String,

    pub software_source_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_software_source_filter: Option<CustomSoftwareSourceFilter>,

    /// Indicates whether the service should automatically resolve package dependencies when including specific packages in the software source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auto_resolve_dependencies: Option<bool>,

    /// Indicates whether the service should create the software source from a list of packages provided by the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_created_from_package_list: Option<bool>,

    /// Indicates whether the software source will include only the latest versions of content from vendor software sources, while accounting for other constraints set in the custom or versioned custom software source (such as a package list or filters). * For a module filter that does not specify a stream, this will include all available streams, and within each stream only the latest version of packages. * For a module filter that does specify a stream, this will include only the latest version of packages for the specified stream. * For a package filter that does not specify a version, this will include only the latest available version of the package. * For a package filter that does specify a version, this will include only the specified version of the package (the isLatestContentOnly attribute is ignored). * For a package list, this will include only the specified version of packages and modules in the list (the isLatestContentOnly attribute is ignored).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_latest_content_only: Option<bool>,

    /// A property used for compatibility only. It doesn't provide a complete list of packages. See {@link #addPackagesToSoftwareSourceDetails(AddPackagesToSoftwareSourceDetailsRequest) addPackagesToSoftwareSourceDetails} for providing the list of packages used to create the software source when isCreatedFromPackageList is set to true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packages: Option<Vec<String>>,

    /// The creation type of a software source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_sub_type: Option<SoftwareSourceSubType>,
}

/// Required fields for CreateVersionedCustomSoftwareSourceDetails
pub struct CreateVersionedCustomSoftwareSourceDetailsRequired {
    /// List of vendor software sources.
    pub vendor_software_sources: Vec<Id>,

    /// The version to assign to this custom software source.
    pub software_source_version: String,

    pub software_source_type: String,
}

impl CreateVersionedCustomSoftwareSourceDetails {
    /// Create a new CreateVersionedCustomSoftwareSourceDetails with required fields
    pub fn new(required: CreateVersionedCustomSoftwareSourceDetailsRequired) -> Self {
        Self {
            vendor_software_sources: required.vendor_software_sources,

            software_source_version: required.software_source_version,

            software_source_type: required.software_source_type,

            custom_software_source_filter: None,

            is_auto_resolve_dependencies: None,

            is_created_from_package_list: None,

            is_latest_content_only: None,

            packages: None,

            software_source_sub_type: None,
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

    /// Set software_source_version
    pub fn set_software_source_version(mut self, value: String) -> Self {
        self.software_source_version = value;
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
}
