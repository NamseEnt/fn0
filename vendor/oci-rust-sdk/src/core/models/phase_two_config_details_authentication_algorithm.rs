use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseTwoConfigDetailsAuthenticationAlgorithm {
    #[serde(rename = "HMAC_SHA2_256_128")]
    HmacSha2256128,

    #[serde(rename = "HMAC_SHA1_128")]
    HmacSha1128,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
