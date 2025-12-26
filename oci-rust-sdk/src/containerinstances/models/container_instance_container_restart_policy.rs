use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerInstanceContainerRestartPolicy {
    #[serde(rename = "ALWAYS")]
    Always,

    #[serde(rename = "NEVER")]
    Never,

    #[serde(rename = "ON_FAILURE")]
    OnFailure,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}

impl fmt::Display for ContainerInstanceContainerRestartPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Always => write!(f, "ALWAYS"),

            Self::Never => write!(f, "NEVER"),

            Self::OnFailure => write!(f, "ON_FAILURE"),

            Self::UnknownValue => write!(f, "UNKNOWN"),
        }
    }
}
