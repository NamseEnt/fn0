use serde::{Deserialize, Serialize};

/// The type of scheduling frequency for the job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleTypes {
    #[serde(rename = "ONETIME")]
    Onetime,

    #[serde(rename = "RECURRING")]
    Recurring,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
