use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An image and shape that are compatible.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageShapeCompatibilityEntry {
    /// The image OCID.
    pub image_id: String,

    /// The shape name.
    pub shape: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_constraints: Option<ImageMemoryConstraints>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocpu_constraints: Option<ImageOcpuConstraints>,
}

/// Required fields for ImageShapeCompatibilityEntry
pub struct ImageShapeCompatibilityEntryRequired {
    /// The image OCID.
    pub image_id: String,

    /// The shape name.
    pub shape: String,
}

impl ImageShapeCompatibilityEntry {
    /// Create a new ImageShapeCompatibilityEntry with required fields
    pub fn new(required: ImageShapeCompatibilityEntryRequired) -> Self {
        Self {
            image_id: required.image_id,

            shape: required.shape,

            memory_constraints: None,

            ocpu_constraints: None,
        }
    }

    /// Set image_id
    pub fn set_image_id(mut self, value: String) -> Self {
        self.image_id = value;
        self
    }

    /// Set shape
    pub fn set_shape(mut self, value: String) -> Self {
        self.shape = value;
        self
    }

    /// Set memory_constraints
    pub fn set_memory_constraints(mut self, value: Option<ImageMemoryConstraints>) -> Self {
        self.memory_constraints = value;
        self
    }

    /// Set ocpu_constraints
    pub fn set_ocpu_constraints(mut self, value: Option<ImageOcpuConstraints>) -> Self {
        self.ocpu_constraints = value;
        self
    }

    /// Set memory_constraints (unwraps Option)
    pub fn with_memory_constraints(mut self, value: ImageMemoryConstraints) -> Self {
        self.memory_constraints = Some(value);
        self
    }

    /// Set ocpu_constraints (unwraps Option)
    pub fn with_ocpu_constraints(mut self, value: ImageOcpuConstraints) -> Self {
        self.ocpu_constraints = Some(value);
        self
    }
}
