use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossConnectStatusEncryptionStatus {
    #[serde(rename = "UP")]
    Up,

    #[serde(rename = "DOWN")]
    Down,

    #[serde(rename = "CIPHER_MISMATCH")]
    CipherMismatch,

    #[serde(rename = "CKN_MISMATCH")]
    CknMismatch,

    #[serde(rename = "CAK_MISMATCH")]
    CakMismatch,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
