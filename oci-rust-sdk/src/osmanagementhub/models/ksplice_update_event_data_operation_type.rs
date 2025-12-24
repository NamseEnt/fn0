use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KspliceUpdateEventDataOperationType {
    #[serde(rename = "UPDATE_KSPLICE_KERNEL")]
    UpdateKspliceKernel,

    #[serde(rename = "UPDATE_KSPLICE_USERSPACE")]
    UpdateKspliceUserspace,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
