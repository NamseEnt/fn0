use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseOneConfigDetailsAuthenticationAlgorithm {
    #[serde(rename = "SHA2_384")]
    Sha2384,

    #[serde(rename = "SHA2_256")]
    Sha2256,

    #[serde(rename = "SHA1_96")]
    Sha196,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
