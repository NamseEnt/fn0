use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Boolean type ImageCapabilitySchemaDescriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BooleanImageCapabilitySchemaDescriptor {
    pub descriptor_type: String,

    /// the default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<bool>,
}

/// Required fields for BooleanImageCapabilitySchemaDescriptor
pub struct BooleanImageCapabilitySchemaDescriptorRequired {
    pub descriptor_type: String,
}

impl BooleanImageCapabilitySchemaDescriptor {
    /// Create a new BooleanImageCapabilitySchemaDescriptor with required fields
    pub fn new(required: BooleanImageCapabilitySchemaDescriptorRequired) -> Self {
        Self {
            descriptor_type: required.descriptor_type,

            default_value: None,
        }
    }

    /// Set default_value
    pub fn set_default_value(mut self, value: Option<bool>) -> Self {
        self.default_value = value;
        self
    }

    /// Set descriptor_type
    pub fn set_descriptor_type(mut self, value: String) -> Self {
        self.descriptor_type = value;
        self
    }

    /// Set default_value (unwraps Option)
    pub fn with_default_value(mut self, value: bool) -> Self {
        self.default_value = Some(value);
        self
    }
}
