use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSourceDetails {
    pub source_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_system: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_system_version: Option<String>,

    /// The format of the image to be imported. Only monolithic images are supported. This attribute is not used for exported Oracle images with the OCI image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_image_type: Option<ImageSourceDetailsSourceImageType>,
}

/// Required fields for ImageSourceDetails
pub struct ImageSourceDetailsRequired {
    pub source_type: String,
}

impl ImageSourceDetails {
    /// Create a new ImageSourceDetails with required fields
    pub fn new(required: ImageSourceDetailsRequired) -> Self {
        Self {
            source_type: required.source_type,

            operating_system: None,

            operating_system_version: None,

            source_image_type: None,
        }
    }

    /// Set operating_system
    pub fn set_operating_system(mut self, value: Option<String>) -> Self {
        self.operating_system = value;
        self
    }

    /// Set operating_system_version
    pub fn set_operating_system_version(mut self, value: Option<String>) -> Self {
        self.operating_system_version = value;
        self
    }

    /// Set source_image_type
    pub fn set_source_image_type(
        mut self,
        value: Option<ImageSourceDetailsSourceImageType>,
    ) -> Self {
        self.source_image_type = value;
        self
    }

    /// Set source_type
    pub fn set_source_type(mut self, value: String) -> Self {
        self.source_type = value;
        self
    }

    /// Set operating_system (unwraps Option)
    pub fn with_operating_system(mut self, value: impl Into<String>) -> Self {
        self.operating_system = Some(value.into());
        self
    }

    /// Set operating_system_version (unwraps Option)
    pub fn with_operating_system_version(mut self, value: impl Into<String>) -> Self {
        self.operating_system_version = Some(value.into());
        self
    }

    /// Set source_image_type (unwraps Option)
    pub fn with_source_image_type(mut self, value: ImageSourceDetailsSourceImageType) -> Self {
        self.source_image_type = Some(value);
        self
    }
}
