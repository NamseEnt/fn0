use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseOneConfigDetailsEncryptionAlgorithm {
    #[serde(rename = "AES_256_CBC")]
    Aes256Cbc,

    #[serde(rename = "AES_192_CBC")]
    Aes192Cbc,

    #[serde(rename = "AES_128_CBC")]
    Aes128Cbc,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
