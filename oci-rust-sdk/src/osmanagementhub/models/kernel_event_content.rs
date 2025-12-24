use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides information collected for the kernel event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KernelEventContent {
    /// Crash content availability status: * 'NOT_AVAILABLE' indicates the content is not available on the instance nor in the service * 'AVAILABLE_ON_INSTANCE' indicates the content is only available on the instance. * 'AVAILABLE_ON_SERVICE' indicates the content is only available on the service. * 'AVAILABLE_ON_INSTANCE_AND_SERVICE' indicates the content is available both on the instance and the service * 'AVAILABLE_ON_INSTANCE_UPLOAD_IN_PROGRESS' indicates the content is available on the instance and its upload to the service is in progress.
    pub content_availability: KernelEventContentContentAvailability,

    /// Location of the Kernel event content.
    pub content_location: String,

    #[serde(rename = "type")]
    pub r#type: String,

    /// Size of the event content. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
}

/// Required fields for KernelEventContent
pub struct KernelEventContentRequired {
    /// Crash content availability status: * 'NOT_AVAILABLE' indicates the content is not available on the instance nor in the service * 'AVAILABLE_ON_INSTANCE' indicates the content is only available on the instance. * 'AVAILABLE_ON_SERVICE' indicates the content is only available on the service. * 'AVAILABLE_ON_INSTANCE_AND_SERVICE' indicates the content is available both on the instance and the service * 'AVAILABLE_ON_INSTANCE_UPLOAD_IN_PROGRESS' indicates the content is available on the instance and its upload to the service is in progress.
    pub content_availability: KernelEventContentContentAvailability,

    /// Location of the Kernel event content.
    pub content_location: String,

    pub r#type: String,
}

impl KernelEventContent {
    /// Create a new KernelEventContent with required fields
    pub fn new(required: KernelEventContentRequired) -> Self {
        Self {
            content_availability: required.content_availability,

            content_location: required.content_location,

            r#type: required.r#type,

            size: None,
        }
    }

    /// Set content_availability
    pub fn set_content_availability(
        mut self,
        value: KernelEventContentContentAvailability,
    ) -> Self {
        self.content_availability = value;
        self
    }

    /// Set content_location
    pub fn set_content_location(mut self, value: String) -> Self {
        self.content_location = value;
        self
    }

    /// Set size
    pub fn set_size(mut self, value: Option<i64>) -> Self {
        self.size = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set size (unwraps Option)
    pub fn with_size(mut self, value: i64) -> Self {
        self.size = Some(value);
        self
    }
}
