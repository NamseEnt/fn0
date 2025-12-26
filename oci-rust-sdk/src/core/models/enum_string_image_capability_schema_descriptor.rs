use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Enum String type of ImageCapabilitySchemaDescriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumStringImageCapabilitySchemaDescriptor {
    /// the list of values for the enum
    pub values: Vec<String>,

    pub descriptor_type: String,

    /// the default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}

/// Required fields for EnumStringImageCapabilitySchemaDescriptor
pub struct EnumStringImageCapabilitySchemaDescriptorRequired {
    /// the list of values for the enum
    pub values: Vec<String>,

    pub descriptor_type: String,
}

impl EnumStringImageCapabilitySchemaDescriptor {
    /// Create a new EnumStringImageCapabilitySchemaDescriptor with required fields
    pub fn new(required: EnumStringImageCapabilitySchemaDescriptorRequired) -> Self {
        Self {
            values: required.values,

            descriptor_type: required.descriptor_type,

            default_value: None,
        }
    }

    /// Set values
    pub fn set_values(mut self, value: Vec<String>) -> Self {
        self.values = value;
        self
    }

    /// Set default_value
    pub fn set_default_value(mut self, value: Option<String>) -> Self {
        self.default_value = value;
        self
    }

    /// Set descriptor_type
    pub fn set_descriptor_type(mut self, value: String) -> Self {
        self.descriptor_type = value;
        self
    }

    /// Set default_value (unwraps Option)
    pub fn with_default_value(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }
}
