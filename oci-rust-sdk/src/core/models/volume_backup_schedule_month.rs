use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeBackupScheduleMonth {
    #[serde(rename = "JANUARY")]
    January,

    #[serde(rename = "FEBRUARY")]
    February,

    #[serde(rename = "MARCH")]
    March,

    #[serde(rename = "APRIL")]
    April,

    #[serde(rename = "MAY")]
    May,

    #[serde(rename = "JUNE")]
    June,

    #[serde(rename = "JULY")]
    July,

    #[serde(rename = "AUGUST")]
    August,

    #[serde(rename = "SEPTEMBER")]
    September,

    #[serde(rename = "OCTOBER")]
    October,

    #[serde(rename = "NOVEMBER")]
    November,

    #[serde(rename = "DECEMBER")]
    December,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
