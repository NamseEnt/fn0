use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The shape that Oracle recommends you to use an alternative to the current shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeAlternativeObject {
    /// The name of the shape.
    pub shape_name: String,
}

/// Required fields for ShapeAlternativeObject
pub struct ShapeAlternativeObjectRequired {
    /// The name of the shape.
    pub shape_name: String,
}

impl ShapeAlternativeObject {
    /// Create a new ShapeAlternativeObject with required fields
    pub fn new(required: ShapeAlternativeObjectRequired) -> Self {
        Self {
            shape_name: required.shape_name,
        }
    }

    /// Set shape_name
    pub fn set_shape_name(mut self, value: String) -> Self {
        self.shape_name = value;
        self
    }
}
