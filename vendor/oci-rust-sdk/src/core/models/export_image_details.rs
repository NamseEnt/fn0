use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The destination details for the image export. <p> Set {@code destinationType} to {@code objectStorageTuple} and use {@link #exportImageViaObjectStorageTupleDetails(ExportImageViaObjectStorageTupleDetailsRequest) exportImageViaObjectStorageTupleDetails} when specifying the namespace, bucket name, and object name. <p> Set {@code destinationType} to {@code objectStorageUri} and use {@link #exportImageViaObjectStorageUriDetails(ExportImageViaObjectStorageUriDetailsRequest) exportImageViaObjectStorageUriDetails} when specifying the Object Storage URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportImageDetails {
    pub destination_type: String,

    /// The format to export the image to. The default value is {@code OCI}. <p> The following image formats are available: <p> - {@code OCI} - Oracle Cloud Infrastructure file with a QCOW2 image and Oracle Cloud Infrastructure metadata (.oci). Use this format to export a custom image that you want to import into other tenancies or regions. - {@code QCOW2} - QEMU Copy On Write (.qcow2) - {@code VDI} - Virtual Disk Image (.vdi) for Oracle VM VirtualBox - {@code VHD} - Virtual Hard Disk (.vhd) for Hyper-V - {@code VMDK} - Virtual Machine Disk (.vmdk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_format: Option<ExportImageDetailsExportFormat>,
}

/// Required fields for ExportImageDetails
pub struct ExportImageDetailsRequired {
    pub destination_type: String,
}

impl ExportImageDetails {
    /// Create a new ExportImageDetails with required fields
    pub fn new(required: ExportImageDetailsRequired) -> Self {
        Self {
            destination_type: required.destination_type,

            export_format: None,
        }
    }

    /// Set export_format
    pub fn set_export_format(mut self, value: Option<ExportImageDetailsExportFormat>) -> Self {
        self.export_format = value;
        self
    }

    /// Set destination_type
    pub fn set_destination_type(mut self, value: String) -> Self {
        self.destination_type = value;
        self
    }

    /// Set export_format (unwraps Option)
    pub fn with_export_format(mut self, value: ExportImageDetailsExportFormat) -> Self {
        self.export_format = Some(value);
        self
    }
}
