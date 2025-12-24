use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportImageDetailsExportFormat {
    #[serde(rename = "QCOW2")]
    Qcow2,

    #[serde(rename = "VMDK")]
    Vmdk,

    #[serde(rename = "OCI")]
    Oci,

    #[serde(rename = "VHD")]
    Vhd,

    #[serde(rename = "VDI")]
    Vdi,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
