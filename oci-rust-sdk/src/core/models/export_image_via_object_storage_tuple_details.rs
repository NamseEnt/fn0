use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportImageViaObjectStorageTupleDetails {
    /// The Object Storage bucket to export the image to.
    pub bucket_name: String,

    /// The Object Storage namespace to export the image to.
    pub namespace_name: String,

    /// The Object Storage object name for the exported image.
    pub object_name: String,

    pub destination_type: String,
}

/// Required fields for ExportImageViaObjectStorageTupleDetails
pub struct ExportImageViaObjectStorageTupleDetailsRequired {
    /// The Object Storage bucket to export the image to.
    pub bucket_name: String,

    /// The Object Storage namespace to export the image to.
    pub namespace_name: String,

    /// The Object Storage object name for the exported image.
    pub object_name: String,

    pub destination_type: String,
}

impl ExportImageViaObjectStorageTupleDetails {
    /// Create a new ExportImageViaObjectStorageTupleDetails with required fields
    pub fn new(required: ExportImageViaObjectStorageTupleDetailsRequired) -> Self {
        Self {
            bucket_name: required.bucket_name,

            namespace_name: required.namespace_name,

            object_name: required.object_name,

            destination_type: required.destination_type,
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

    /// Set destination_type
    pub fn set_destination_type(mut self, value: String) -> Self {
        self.destination_type = value;
        self
    }
}
