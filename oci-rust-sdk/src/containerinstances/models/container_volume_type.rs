use serde::{Deserialize, Serialize};

/// Container Volume Type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerVolumeType {
    #[serde(rename = "EMPTYDIR")]
    Emptydir,

    #[serde(rename = "CONFIGFILE")]
    Configfile,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
