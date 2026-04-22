use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides identifying information for the specified software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareSourceDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source.
    pub id: String,

    /// Software source name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Software source description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Type of the software source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_type: Option<SoftwareSourceType>,

    /// Indicates whether this is a required software source for Autonomous Linux instances. If true, the user can't unselect it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mandatory_for_autonomous_linux: Option<bool>,
}

/// Required fields for SoftwareSourceDetails
pub struct SoftwareSourceDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source.
    pub id: String,
}

impl SoftwareSourceDetails {
    /// Create a new SoftwareSourceDetails with required fields
    pub fn new(required: SoftwareSourceDetailsRequired) -> Self {
        Self {
            id: required.id,

            display_name: None,

            description: None,

            software_source_type: None,

            is_mandatory_for_autonomous_linux: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set software_source_type
    pub fn set_software_source_type(mut self, value: Option<SoftwareSourceType>) -> Self {
        self.software_source_type = value;
        self
    }

    /// Set is_mandatory_for_autonomous_linux
    pub fn set_is_mandatory_for_autonomous_linux(mut self, value: Option<bool>) -> Self {
        self.is_mandatory_for_autonomous_linux = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set software_source_type (unwraps Option)
    pub fn with_software_source_type(mut self, value: SoftwareSourceType) -> Self {
        self.software_source_type = Some(value);
        self
    }

    /// Set is_mandatory_for_autonomous_linux (unwraps Option)
    pub fn with_is_mandatory_for_autonomous_linux(mut self, value: bool) -> Self {
        self.is_mandatory_for_autonomous_linux = Some(value);
        self
    }
}
