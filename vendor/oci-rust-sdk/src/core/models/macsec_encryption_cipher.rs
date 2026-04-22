use serde::{Deserialize, Serialize};

/// Type of encryption cipher suite to use for the MACsec connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacsecEncryptionCipher {
    #[serde(rename = "AES128_GCM")]
    Aes128Gcm,

    #[serde(rename = "AES128_GCM_XPN")]
    Aes128GcmXpn,

    #[serde(rename = "AES256_GCM")]
    Aes256Gcm,

    #[serde(rename = "AES256_GCM_XPN")]
    Aes256GcmXpn,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
