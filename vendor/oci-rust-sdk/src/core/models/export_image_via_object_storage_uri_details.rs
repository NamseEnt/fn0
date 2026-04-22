use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportImageViaObjectStorageUriDetails {
    /// The Object Storage URL to export the image to. See [Object Storage URLs](https://docs.oracle.com/iaas/Content/Compute/Tasks/imageimportexport.htm#URLs) and [Using Pre-Authenticated Requests](https://docs.oracle.com/iaas/Content/Object/Tasks/usingpreauthenticatedrequests.htm) for constructing URLs for image import/export.
    pub destination_uri: String,

    pub destination_type: String,
}

/// Required fields for ExportImageViaObjectStorageUriDetails
pub struct ExportImageViaObjectStorageUriDetailsRequired {
    /// The Object Storage URL to export the image to. See [Object Storage URLs](https://docs.oracle.com/iaas/Content/Compute/Tasks/imageimportexport.htm#URLs) and [Using Pre-Authenticated Requests](https://docs.oracle.com/iaas/Content/Object/Tasks/usingpreauthenticatedrequests.htm) for constructing URLs for image import/export.
    pub destination_uri: String,

    pub destination_type: String,
}

impl ExportImageViaObjectStorageUriDetails {
    /// Create a new ExportImageViaObjectStorageUriDetails with required fields
    pub fn new(required: ExportImageViaObjectStorageUriDetailsRequired) -> Self {
        Self {
            destination_uri: required.destination_uri,

            destination_type: required.destination_type,
        }
    }

    /// Set destination_uri
    pub fn set_destination_uri(mut self, value: String) -> Self {
        self.destination_uri = value;
        self
    }

    /// Set destination_type
    pub fn set_destination_type(mut self, value: String) -> Self {
        self.destination_type = value;
        self
    }
}
