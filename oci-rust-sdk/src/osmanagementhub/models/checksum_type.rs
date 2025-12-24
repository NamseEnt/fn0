use serde::{Deserialize, Serialize};

/// Type of checksum used for the software source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChecksumType {
    #[serde(rename = "SHA1")]
    Sha1,

    #[serde(rename = "SHA256")]
    Sha256,

    #[serde(rename = "SHA384")]
    Sha384,

    #[serde(rename = "SHA512")]
    Sha512,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
