use serde::{Deserialize, Serialize};

/// Possible erratum advisory types. These map to ELSA, ELBA, ELEA types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdvisoryTypes {
    #[serde(rename = "SECURITY")]
    Security,

    #[serde(rename = "BUGFIX")]
    Bugfix,

    #[serde(rename = "ENHANCEMENT")]
    Enhancement,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
