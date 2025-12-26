use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Image Capability Schema Descriptor is a type of capability for an image.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageCapabilitySchemaDescriptor {
    pub source: ImageCapabilitySchemaDescriptorSource,

    pub descriptor_type: String,
}

/// Required fields for ImageCapabilitySchemaDescriptor
pub struct ImageCapabilitySchemaDescriptorRequired {
    pub source: ImageCapabilitySchemaDescriptorSource,

    pub descriptor_type: String,
}

impl ImageCapabilitySchemaDescriptor {
    /// Create a new ImageCapabilitySchemaDescriptor with required fields
    pub fn new(required: ImageCapabilitySchemaDescriptorRequired) -> Self {
        Self {
            source: required.source,

            descriptor_type: required.descriptor_type,
        }
    }

    /// Set source
    pub fn set_source(mut self, value: ImageCapabilitySchemaDescriptorSource) -> Self {
        self.source = value;
        self
    }

    /// Set descriptor_type
    pub fn set_descriptor_type(mut self, value: String) -> Self {
        self.descriptor_type = value;
        self
    }
}
