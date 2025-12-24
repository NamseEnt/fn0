use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An object that defines a module stream provided by a software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleStream {
    /// The name of the module that contains the stream.
    pub module_name: String,

    /// The name of the stream.
    pub name: String,

    /// Indicates if this stream is the default for its module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source that provides this module stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_id: Option<String>,

    /// The architecture for which the packages in this module stream were built.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch_type: Option<ArchType>,

    /// A description of the contents of the module stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A list of profiles that are part of the stream.  Each element in the list is the name of a profile.  The name is suitable to use as an argument to other OS Management Hub APIs that interact directly with module stream profiles.  However, it is not URL encoded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<Vec<String>>,

    /// A list of packages that are contained by the stream.  Each element in the list is the name of a package.  The name is suitable to use as an argument to other OS Management Hub APIs that interact directly with packages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packages: Option<Vec<String>>,

    /// Indicates whether this module stream is the latest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_latest: Option<bool>,
}

/// Required fields for ModuleStream
pub struct ModuleStreamRequired {
    /// The name of the module that contains the stream.
    pub module_name: String,

    /// The name of the stream.
    pub name: String,
}

impl ModuleStream {
    /// Create a new ModuleStream with required fields
    pub fn new(required: ModuleStreamRequired) -> Self {
        Self {
            module_name: required.module_name,

            name: required.name,

            is_default: None,

            software_source_id: None,

            arch_type: None,

            description: None,

            profiles: None,

            packages: None,

            is_latest: None,
        }
    }

    /// Set module_name
    pub fn set_module_name(mut self, value: String) -> Self {
        self.module_name = value;
        self
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set is_default
    pub fn set_is_default(mut self, value: Option<bool>) -> Self {
        self.is_default = value;
        self
    }

    /// Set software_source_id
    pub fn set_software_source_id(mut self, value: Option<String>) -> Self {
        self.software_source_id = value;
        self
    }

    /// Set arch_type
    pub fn set_arch_type(mut self, value: Option<ArchType>) -> Self {
        self.arch_type = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set profiles
    pub fn set_profiles(mut self, value: Option<Vec<String>>) -> Self {
        self.profiles = value;
        self
    }

    /// Set packages
    pub fn set_packages(mut self, value: Option<Vec<String>>) -> Self {
        self.packages = value;
        self
    }

    /// Set is_latest
    pub fn set_is_latest(mut self, value: Option<bool>) -> Self {
        self.is_latest = value;
        self
    }

    /// Set is_default (unwraps Option)
    pub fn with_is_default(mut self, value: bool) -> Self {
        self.is_default = Some(value);
        self
    }

    /// Set software_source_id (unwraps Option)
    pub fn with_software_source_id(mut self, value: impl Into<String>) -> Self {
        self.software_source_id = Some(value.into());
        self
    }

    /// Set arch_type (unwraps Option)
    pub fn with_arch_type(mut self, value: ArchType) -> Self {
        self.arch_type = Some(value);
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set profiles (unwraps Option)
    pub fn with_profiles(mut self, value: Vec<String>) -> Self {
        self.profiles = Some(value);
        self
    }

    /// Set packages (unwraps Option)
    pub fn with_packages(mut self, value: Vec<String>) -> Self {
        self.packages = Some(value);
        self
    }

    /// Set is_latest (unwraps Option)
    pub fn with_is_latest(mut self, value: bool) -> Self {
        self.is_latest = Some(value);
        self
    }
}
