use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminateInstanceRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance.
    pub instance_id: String,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,

    /// Specifies whether to delete or preserve the boot volume when terminating an instance. When set to {@code true}, the boot volume is preserved. The default value is {@code false}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_boot_volume: Option<bool>,

    /// Specifies whether to delete or preserve the data volumes created during launch when terminating an instance. When set to {@code true}, the data volumes are preserved. The default value is {@code true}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_data_volumes_created_at_launch: Option<bool>,

    /// This optional parameter overrides recycle level for hosts. The parameter can be used when hosts are associated with a Capacity Reservation. * {@code FULL_RECYCLE} - Does not skip host wipe. This is the default behavior.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recycle_level: Option<TerminateInstanceRequestRecycleLevel>,
}


/// Required fields for TerminateInstanceRequest
pub struct TerminateInstanceRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance.
    pub instance_id: String,
}

impl TerminateInstanceRequest {
    /// Create a new TerminateInstanceRequest with required fields
    pub fn new(required: TerminateInstanceRequestRequired) -> Self {
        Self {
            instance_id: required.instance_id,

            if_match: None,

            preserve_boot_volume: None,

            preserve_data_volumes_created_at_launch: None,

            recycle_level: None,
}
    }

    /// Set instance_id
    pub fn set_instance_id(mut self, value: String) -> Self {
        self.instance_id = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set preserve_boot_volume
    pub fn set_preserve_boot_volume(mut self, value: Option<bool>) -> Self {
        self.preserve_boot_volume = value;
        self
    }

    /// Set preserve_data_volumes_created_at_launch
    pub fn set_preserve_data_volumes_created_at_launch(mut self, value: Option<bool>) -> Self {
        self.preserve_data_volumes_created_at_launch = value;
        self
    }

    /// Set recycle_level
    pub fn set_recycle_level(mut self, value: Option<TerminateInstanceRequestRecycleLevel>) -> Self {
        self.recycle_level = value;
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }

    /// Set preserve_boot_volume (unwraps Option)
    pub fn with_preserve_boot_volume(mut self, value: bool) -> Self {
        self.preserve_boot_volume = Some(value);
        self
    }

    /// Set preserve_data_volumes_created_at_launch (unwraps Option)
    pub fn with_preserve_data_volumes_created_at_launch(mut self, value: bool) -> Self {
        self.preserve_data_volumes_created_at_launch = Some(value);
        self
    }

    /// Set recycle_level (unwraps Option)
    pub fn with_recycle_level(mut self, value: TerminateInstanceRequestRecycleLevel) -> Self {
        self.recycle_level = Some(value);
        self
    }
}


