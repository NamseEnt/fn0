use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Defines the backup frequency and retention period for a volume backup policy. For more information, see [Policy-Based Backups](https://docs.oracle.com/iaas/Content/Block/Tasks/schedulingvolumebackups.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeBackupSchedule {
    /// The type of volume backup to create.
    pub backup_type: VolumeBackupScheduleBackupType,

    /// The volume backup frequency.
    pub period: VolumeBackupSchedulePeriod,

    /// How long, in seconds, to keep the volume backups created by this schedule. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub retention_seconds: i64,

    /// The number of seconds that the volume backup start time should be shifted from the default interval boundaries specified by the period. The volume backup start time is the frequency start time plus the offset. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_seconds: Option<i64>,

    /// Indicates how the offset is defined. If value is {@code STRUCTURED}, then {@code hourOfDay}, {@code dayOfWeek}, {@code dayOfMonth}, and {@code month} fields are used and {@code offsetSeconds} will be ignored in requests and users should ignore its value from the responses. <p> {@code hourOfDay} is applicable for periods {@code ONE_DAY}, {@code ONE_WEEK}, {@code ONE_MONTH} and {@code ONE_YEAR}. <p> {@code dayOfWeek} is applicable for period {@code ONE_WEEK}. <p> {@code dayOfMonth} is applicable for periods {@code ONE_MONTH} and {@code ONE_YEAR}. <p> 'month' is applicable for period 'ONE_YEAR'. <p> They will be ignored in the requests for inapplicable periods. <p> If value is {@code NUMERIC_SECONDS}, then {@code offsetSeconds} will be used for both requests and responses and the structured fields will be ignored in the requests and users should ignore their values from the responses. <p> For clients using older versions of Apis and not sending {@code offsetType} in their requests, the behaviour is just like {@code NUMERIC_SECONDS}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_type: Option<VolumeBackupScheduleOffsetType>,

    /// The hour of the day to schedule the volume backup. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hour_of_day: Option<i64>,

    /// The day of the week to schedule the volume backup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_of_week: Option<VolumeBackupScheduleDayOfWeek>,

    /// The day of the month to schedule the volume backup. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_of_month: Option<i64>,

    /// The month of the year to schedule the volume backup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub month: Option<VolumeBackupScheduleMonth>,

    /// Specifies what time zone is the schedule in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<VolumeBackupScheduleTimeZone>,
}

/// Required fields for VolumeBackupSchedule
pub struct VolumeBackupScheduleRequired {
    /// The type of volume backup to create.
    pub backup_type: VolumeBackupScheduleBackupType,

    /// The volume backup frequency.
    pub period: VolumeBackupSchedulePeriod,

    /// How long, in seconds, to keep the volume backups created by this schedule. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub retention_seconds: i64,
}

impl VolumeBackupSchedule {
    /// Create a new VolumeBackupSchedule with required fields
    pub fn new(required: VolumeBackupScheduleRequired) -> Self {
        Self {
            backup_type: required.backup_type,

            period: required.period,

            retention_seconds: required.retention_seconds,

            offset_seconds: None,

            offset_type: None,

            hour_of_day: None,

            day_of_week: None,

            day_of_month: None,

            month: None,

            time_zone: None,
        }
    }

    /// Set backup_type
    pub fn set_backup_type(mut self, value: VolumeBackupScheduleBackupType) -> Self {
        self.backup_type = value;
        self
    }

    /// Set offset_seconds
    pub fn set_offset_seconds(mut self, value: Option<i64>) -> Self {
        self.offset_seconds = value;
        self
    }

    /// Set period
    pub fn set_period(mut self, value: VolumeBackupSchedulePeriod) -> Self {
        self.period = value;
        self
    }

    /// Set offset_type
    pub fn set_offset_type(mut self, value: Option<VolumeBackupScheduleOffsetType>) -> Self {
        self.offset_type = value;
        self
    }

    /// Set hour_of_day
    pub fn set_hour_of_day(mut self, value: Option<i64>) -> Self {
        self.hour_of_day = value;
        self
    }

    /// Set day_of_week
    pub fn set_day_of_week(mut self, value: Option<VolumeBackupScheduleDayOfWeek>) -> Self {
        self.day_of_week = value;
        self
    }

    /// Set day_of_month
    pub fn set_day_of_month(mut self, value: Option<i64>) -> Self {
        self.day_of_month = value;
        self
    }

    /// Set month
    pub fn set_month(mut self, value: Option<VolumeBackupScheduleMonth>) -> Self {
        self.month = value;
        self
    }

    /// Set retention_seconds
    pub fn set_retention_seconds(mut self, value: i64) -> Self {
        self.retention_seconds = value;
        self
    }

    /// Set time_zone
    pub fn set_time_zone(mut self, value: Option<VolumeBackupScheduleTimeZone>) -> Self {
        self.time_zone = value;
        self
    }

    /// Set offset_seconds (unwraps Option)
    pub fn with_offset_seconds(mut self, value: i64) -> Self {
        self.offset_seconds = Some(value);
        self
    }

    /// Set offset_type (unwraps Option)
    pub fn with_offset_type(mut self, value: VolumeBackupScheduleOffsetType) -> Self {
        self.offset_type = Some(value);
        self
    }

    /// Set hour_of_day (unwraps Option)
    pub fn with_hour_of_day(mut self, value: i64) -> Self {
        self.hour_of_day = Some(value);
        self
    }

    /// Set day_of_week (unwraps Option)
    pub fn with_day_of_week(mut self, value: VolumeBackupScheduleDayOfWeek) -> Self {
        self.day_of_week = Some(value);
        self
    }

    /// Set day_of_month (unwraps Option)
    pub fn with_day_of_month(mut self, value: i64) -> Self {
        self.day_of_month = Some(value);
        self
    }

    /// Set month (unwraps Option)
    pub fn with_month(mut self, value: VolumeBackupScheduleMonth) -> Self {
        self.month = Some(value);
        self
    }

    /// Set time_zone (unwraps Option)
    pub fn with_time_zone(mut self, value: VolumeBackupScheduleTimeZone) -> Self {
        self.time_zone = Some(value);
        self
    }
}
