use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Total CPU and memory capacity for each capacity bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapacityBinPreview {
    /// Zero-based index for the corresponding capacity bucket. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub capacity_index: i64,

    /// The total OCPUs of the capacity bucket. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub total_ocpus: i64,

    /// The total memory of the capacity bucket, in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub total_memory_in_g_bs: i64,

    /// List of VMI shapes supported on each capacity bucket.
    pub supported_shapes: Vec<String>,
}

/// Required fields for CapacityBinPreview
pub struct CapacityBinPreviewRequired {
    /// Zero-based index for the corresponding capacity bucket. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub capacity_index: i64,

    /// The total OCPUs of the capacity bucket. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub total_ocpus: i64,

    /// The total memory of the capacity bucket, in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub total_memory_in_g_bs: i64,

    /// List of VMI shapes supported on each capacity bucket.
    pub supported_shapes: Vec<String>,
}

impl CapacityBinPreview {
    /// Create a new CapacityBinPreview with required fields
    pub fn new(required: CapacityBinPreviewRequired) -> Self {
        Self {
            capacity_index: required.capacity_index,

            total_ocpus: required.total_ocpus,

            total_memory_in_g_bs: required.total_memory_in_g_bs,

            supported_shapes: required.supported_shapes,
        }
    }

    /// Set capacity_index
    pub fn set_capacity_index(mut self, value: i64) -> Self {
        self.capacity_index = value;
        self
    }

    /// Set total_ocpus
    pub fn set_total_ocpus(mut self, value: i64) -> Self {
        self.total_ocpus = value;
        self
    }

    /// Set total_memory_in_g_bs
    pub fn set_total_memory_in_g_bs(mut self, value: i64) -> Self {
        self.total_memory_in_g_bs = value;
        self
    }

    /// Set supported_shapes
    pub fn set_supported_shapes(mut self, value: Vec<String>) -> Self {
        self.supported_shapes = value;
        self
    }
}
