use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information about a module which is provided by a software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleSummary {
    /// The name of the module.
    pub name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source.
    pub software_source_id: String,

    /// List of stream names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streams: Option<Vec<String>>,
}

/// Required fields for ModuleSummary
pub struct ModuleSummaryRequired {
    /// The name of the module.
    pub name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source.
    pub software_source_id: String,
}

impl ModuleSummary {
    /// Create a new ModuleSummary with required fields
    pub fn new(required: ModuleSummaryRequired) -> Self {
        Self {
            name: required.name,

            software_source_id: required.software_source_id,

            streams: None,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set streams
    pub fn set_streams(mut self, value: Option<Vec<String>>) -> Self {
        self.streams = value;
        self
    }

    /// Set software_source_id
    pub fn set_software_source_id(mut self, value: String) -> Self {
        self.software_source_id = value;
        self
    }

    /// Set streams (unwraps Option)
    pub fn with_streams(mut self, value: Vec<String>) -> Self {
        self.streams = Some(value);
        self
    }
}
