use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to update a module stream profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleStreamProfileDetails {
    /// The name of a module.
    pub module_name: String,

    /// The name of a stream of the specified module.
    pub stream_name: String,

    /// The name of a profile of the specified module stream.
    pub profile_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source that contains the module stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_id: Option<String>,
}

/// Required fields for ModuleStreamProfileDetails
pub struct ModuleStreamProfileDetailsRequired {
    /// The name of a module.
    pub module_name: String,

    /// The name of a stream of the specified module.
    pub stream_name: String,

    /// The name of a profile of the specified module stream.
    pub profile_name: String,
}

impl ModuleStreamProfileDetails {
    /// Create a new ModuleStreamProfileDetails with required fields
    pub fn new(required: ModuleStreamProfileDetailsRequired) -> Self {
        Self {
            module_name: required.module_name,

            stream_name: required.stream_name,

            profile_name: required.profile_name,

            software_source_id: None,
        }
    }

    /// Set module_name
    pub fn set_module_name(mut self, value: String) -> Self {
        self.module_name = value;
        self
    }

    /// Set stream_name
    pub fn set_stream_name(mut self, value: String) -> Self {
        self.stream_name = value;
        self
    }

    /// Set profile_name
    pub fn set_profile_name(mut self, value: String) -> Self {
        self.profile_name = value;
        self
    }

    /// Set software_source_id
    pub fn set_software_source_id(mut self, value: Option<String>) -> Self {
        self.software_source_id = value;
        self
    }

    /// Set software_source_id (unwraps Option)
    pub fn with_software_source_id(mut self, value: impl Into<String>) -> Self {
        self.software_source_id = Some(value.into());
        self
    }
}
