use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Defines the mapping from volume to a mount path in a container.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVolumeMountDetails {
    /// The volume access path.
    pub mount_path: String,

    /// The name of the volume. Avoid entering confidential information.
    pub volume_name: String,

    /// A subpath inside the referenced volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_path: Option<String>,

    /// Whether the volume was mounted in read-only mode. By default, the volume is not read-only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_read_only: Option<bool>,

    /// If there is more than one partition in the volume, reference this number of partitions. Here is an example: Number  Start   End     Size    File system  Name                  Flags 1      1049kB  106MB   105MB   fat16        EFI System Partition  boot, esp 2      106MB   1180MB  1074MB  xfs 3      1180MB  50.0GB  48.8GB                                     lvm Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition: Option<i64>,
}

/// Required fields for CreateVolumeMountDetails
pub struct CreateVolumeMountDetailsRequired {
    /// The volume access path.
    pub mount_path: String,

    /// The name of the volume. Avoid entering confidential information.
    pub volume_name: String,
}

impl CreateVolumeMountDetails {
    /// Create a new CreateVolumeMountDetails with required fields
    pub fn new(required: CreateVolumeMountDetailsRequired) -> Self {
        Self {
            mount_path: required.mount_path,

            volume_name: required.volume_name,

            sub_path: None,

            is_read_only: None,

            partition: None,
        }
    }

    /// Set mount_path
    pub fn set_mount_path(mut self, value: String) -> Self {
        self.mount_path = value;
        self
    }

    /// Set volume_name
    pub fn set_volume_name(mut self, value: String) -> Self {
        self.volume_name = value;
        self
    }

    /// Set sub_path
    pub fn set_sub_path(mut self, value: Option<String>) -> Self {
        self.sub_path = value;
        self
    }

    /// Set is_read_only
    pub fn set_is_read_only(mut self, value: Option<bool>) -> Self {
        self.is_read_only = value;
        self
    }

    /// Set partition
    pub fn set_partition(mut self, value: Option<i64>) -> Self {
        self.partition = value;
        self
    }

    /// Set sub_path (unwraps Option)
    pub fn with_sub_path(mut self, value: impl Into<String>) -> Self {
        self.sub_path = Some(value.into());
        self
    }

    /// Set is_read_only (unwraps Option)
    pub fn with_is_read_only(mut self, value: bool) -> Self {
        self.is_read_only = Some(value);
        self
    }

    /// Set partition (unwraps Option)
    pub fn with_partition(mut self, value: i64) -> Self {
        self.partition = Some(value);
        self
    }
}
