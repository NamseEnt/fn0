use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LetterOfAuthorityCircuitType {
    #[serde(rename = "Single_mode_LC")]
    SingleModeLc,

    #[serde(rename = "Single_mode_SC")]
    SingleModeSc,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
