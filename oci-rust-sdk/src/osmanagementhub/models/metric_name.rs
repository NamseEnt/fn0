use serde::{Deserialize, Serialize};

/// Possible metric names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricName {
    #[serde(rename = "TOTAL_INSTANCE_COUNT")]
    TotalInstanceCount,

    #[serde(rename = "INSTANCE_WITH_AVAILABLE_SECURITY_UPDATES_COUNT")]
    InstanceWithAvailableSecurityUpdatesCount,

    #[serde(rename = "INSTANCE_WITH_AVAILABLE_BUGFIX_UPDATES_COUNT")]
    InstanceWithAvailableBugfixUpdatesCount,

    #[serde(rename = "NORMAL_INSTANCE_COUNT")]
    NormalInstanceCount,

    #[serde(rename = "ERROR_INSTANCE_COUNT")]
    ErrorInstanceCount,

    #[serde(rename = "WARNING_INSTANCE_COUNT")]
    WarningInstanceCount,

    #[serde(rename = "UNREACHABLE_INSTANCE_COUNT")]
    UnreachableInstanceCount,

    #[serde(rename = "REGISTRATION_FAILED_INSTANCE_COUNT")]
    RegistrationFailedInstanceCount,

    #[serde(rename = "DELETING_INSTANCE_COUNT")]
    DeletingInstanceCount,

    #[serde(rename = "ONBOARDING_INSTANCE_COUNT")]
    OnboardingInstanceCount,

    #[serde(rename = "INSTANCE_SECURITY_UPDATES_COUNT")]
    InstanceSecurityUpdatesCount,

    #[serde(rename = "INSTANCE_BUGFIX_UPDATES_COUNT")]
    InstanceBugfixUpdatesCount,

    #[serde(rename = "INSTANCE_SECURITY_ADVISORY_COUNT")]
    InstanceSecurityAdvisoryCount,

    #[serde(rename = "INSTANCE_BUGFIX_ADVISORY_COUNT")]
    InstanceBugfixAdvisoryCount,

    #[serde(rename = "REBOOTING_INSTANCE_COUNT")]
    RebootingInstanceCount,

    #[serde(rename = "NEEDS_REBOOTING_INSTANCE_COUNT")]
    NeedsRebootingInstanceCount,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
