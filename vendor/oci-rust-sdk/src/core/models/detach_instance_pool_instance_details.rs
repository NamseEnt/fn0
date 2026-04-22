use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An instance that is to be detached from an instance pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetachInstancePoolInstanceDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance.
    pub instance_id: String,

    /// Whether to decrease the size of the instance pool when the instance is detached. If {@code true}, the pool size is decreased. If {@code false}, the pool will provision a new, replacement instance using the pool's instance configuration as a template. Default is {@code true}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_decrement_size: Option<bool>,

    /// Whether to permanently terminate (delete) the instance and its attached boot volume when detaching it from the instance pool. Default is {@code false}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auto_terminate: Option<bool>,
}

/// Required fields for DetachInstancePoolInstanceDetails
pub struct DetachInstancePoolInstanceDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance.
    pub instance_id: String,
}

impl DetachInstancePoolInstanceDetails {
    /// Create a new DetachInstancePoolInstanceDetails with required fields
    pub fn new(required: DetachInstancePoolInstanceDetailsRequired) -> Self {
        Self {
            instance_id: required.instance_id,

            is_decrement_size: None,

            is_auto_terminate: None,
        }
    }

    /// Set instance_id
    pub fn set_instance_id(mut self, value: String) -> Self {
        self.instance_id = value;
        self
    }

    /// Set is_decrement_size
    pub fn set_is_decrement_size(mut self, value: Option<bool>) -> Self {
        self.is_decrement_size = value;
        self
    }

    /// Set is_auto_terminate
    pub fn set_is_auto_terminate(mut self, value: Option<bool>) -> Self {
        self.is_auto_terminate = value;
        self
    }

    /// Set is_decrement_size (unwraps Option)
    pub fn with_is_decrement_size(mut self, value: bool) -> Self {
        self.is_decrement_size = Some(value);
        self
    }

    /// Set is_auto_terminate (unwraps Option)
    pub fn with_is_auto_terminate(mut self, value: bool) -> Self {
        self.is_auto_terminate = Some(value);
        self
    }
}
