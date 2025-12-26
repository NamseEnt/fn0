use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Enum Integer type CapabilityDescriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumIntegerImageCapabilityDescriptor {
    /// the list of values for the enum
    pub values: Vec<i64>,

    pub descriptor_type: String,

    /// the default value Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<i64>,
}

/// Required fields for EnumIntegerImageCapabilityDescriptor
pub struct EnumIntegerImageCapabilityDescriptorRequired {
    /// the list of values for the enum
    pub values: Vec<i64>,

    pub descriptor_type: String,
}

impl EnumIntegerImageCapabilityDescriptor {
    /// Create a new EnumIntegerImageCapabilityDescriptor with required fields
    pub fn new(required: EnumIntegerImageCapabilityDescriptorRequired) -> Self {
        Self {
            values: required.values,

            descriptor_type: required.descriptor_type,

            default_value: None,
        }
    }

    /// Set values
    pub fn set_values(mut self, value: Vec<i64>) -> Self {
        self.values = value;
        self
    }

    /// Set default_value
    pub fn set_default_value(mut self, value: Option<i64>) -> Self {
        self.default_value = value;
        self
    }

    /// Set descriptor_type
    pub fn set_descriptor_type(mut self, value: String) -> Self {
        self.descriptor_type = value;
        self
    }

    /// Set default_value (unwraps Option)
    pub fn with_default_value(mut self, value: i64) -> Self {
        self.default_value = Some(value);
        self
    }
}
