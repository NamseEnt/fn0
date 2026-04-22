use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageSourceDetailsSourceImageType {
    #[serde(rename = "QCOW2")]
    Qcow2,

    #[serde(rename = "VMDK")]
    Vmdk,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
