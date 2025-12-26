use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSourceViaObjectStorageUriDetails {
    /// The Object Storage URL for the image.
    pub source_uri: String,

    pub source_type: String,
}

/// Required fields for ImageSourceViaObjectStorageUriDetails
pub struct ImageSourceViaObjectStorageUriDetailsRequired {
    /// The Object Storage URL for the image.
    pub source_uri: String,

    pub source_type: String,
}

impl ImageSourceViaObjectStorageUriDetails {
    /// Create a new ImageSourceViaObjectStorageUriDetails with required fields
    pub fn new(required: ImageSourceViaObjectStorageUriDetailsRequired) -> Self {
        Self {
            source_uri: required.source_uri,

            source_type: required.source_type,
        }
    }

    /// Set source_uri
    pub fn set_source_uri(mut self, value: String) -> Self {
        self.source_uri = value;
        self
    }

    /// Set source_type
    pub fn set_source_type(mut self, value: String) -> Self {
        self.source_type = value;
        self
    }
}
