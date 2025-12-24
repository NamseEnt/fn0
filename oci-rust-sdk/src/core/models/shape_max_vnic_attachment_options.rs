use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// For a flexible shape, the number of VNIC attachments that are available for instances that use this shape. <p> If this field is null, then this shape has a fixed maximum number of VNIC attachments equal to {@code maxVnicAttachments}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeMaxVnicAttachmentOptions {
    /// The lowest maximum value of VNIC attachments. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<i64>,

    /// The highest maximum value of VNIC attachments. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<i64>,

    /// The default number of VNIC attachments allowed per OCPU. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_per_ocpu: Option<i64>,
}

impl ShapeMaxVnicAttachmentOptions {
    /// Create a new ShapeMaxVnicAttachmentOptions
    pub fn new() -> Self {
        Self {
            min: None,

            max: None,

            default_per_ocpu: None,
        }
    }

    /// Set min
    pub fn set_min(mut self, value: Option<i64>) -> Self {
        self.min = value;
        self
    }

    /// Set max
    pub fn set_max(mut self, value: Option<i64>) -> Self {
        self.max = value;
        self
    }

    /// Set default_per_ocpu
    pub fn set_default_per_ocpu(mut self, value: Option<i64>) -> Self {
        self.default_per_ocpu = value;
        self
    }

    /// Set min (unwraps Option)
    pub fn with_min(mut self, value: i64) -> Self {
        self.min = Some(value);
        self
    }

    /// Set max (unwraps Option)
    pub fn with_max(mut self, value: i64) -> Self {
        self.max = Some(value);
        self
    }

    /// Set default_per_ocpu (unwraps Option)
    pub fn with_default_per_ocpu(mut self, value: i64) -> Self {
        self.default_per_ocpu = Some(value);
        self
    }
}

impl Default for ShapeMaxVnicAttachmentOptions {
    fn default() -> Self {
        Self::new()
    }
}
