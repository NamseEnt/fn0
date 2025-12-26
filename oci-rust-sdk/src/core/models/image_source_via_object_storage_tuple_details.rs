use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSourceViaObjectStorageTupleDetails {
    /// The Object Storage bucket for the image.
    pub bucket_name: String,

    /// The Object Storage namespace for the image.
    pub namespace_name: String,

    /// The Object Storage name for the image.
    pub object_name: String,

    pub source_type: String,
}

/// Required fields for ImageSourceViaObjectStorageTupleDetails
pub struct ImageSourceViaObjectStorageTupleDetailsRequired {
    /// The Object Storage bucket for the image.
    pub bucket_name: String,

    /// The Object Storage namespace for the image.
    pub namespace_name: String,

    /// The Object Storage name for the image.
    pub object_name: String,

    pub source_type: String,
}

impl ImageSourceViaObjectStorageTupleDetails {
    /// Create a new ImageSourceViaObjectStorageTupleDetails with required fields
    pub fn new(required: ImageSourceViaObjectStorageTupleDetailsRequired) -> Self {
        Self {
            bucket_name: required.bucket_name,

            namespace_name: required.namespace_name,

            object_name: required.object_name,

            source_type: required.source_type,
        }
    }

    /// Set bucket_name
    pub fn set_bucket_name(mut self, value: String) -> Self {
        self.bucket_name = value;
        self
    }

    /// Set namespace_name
    pub fn set_namespace_name(mut self, value: String) -> Self {
        self.namespace_name = value;
        self
    }

    /// Set object_name
    pub fn set_object_name(mut self, value: String) -> Self {
        self.object_name = value;
        self
    }

    /// Set source_type
    pub fn set_source_type(mut self, value: String) -> Self {
        self.source_type = value;
        self
    }
}
