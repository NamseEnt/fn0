use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The shape configuration for a shape in a capacity report.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapacityReportInstanceShapeConfig {
    /// The total number of OCPUs available to the instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocpus: Option<i64>,

    /// The total amount of memory available to the instance, in gigabytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_in_g_bs: Option<i64>,

    /// The number of NVMe drives to be used for storage. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nvmes: Option<i64>,
}

impl CapacityReportInstanceShapeConfig {
    /// Create a new CapacityReportInstanceShapeConfig
    pub fn new() -> Self {
        Self {
            ocpus: None,

            memory_in_g_bs: None,

            nvmes: None,
        }
    }

    /// Set ocpus
    pub fn set_ocpus(mut self, value: Option<i64>) -> Self {
        self.ocpus = value;
        self
    }

    /// Set memory_in_g_bs
    pub fn set_memory_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.memory_in_g_bs = value;
        self
    }

    /// Set nvmes
    pub fn set_nvmes(mut self, value: Option<i64>) -> Self {
        self.nvmes = value;
        self
    }

    /// Set ocpus (unwraps Option)
    pub fn with_ocpus(mut self, value: i64) -> Self {
        self.ocpus = Some(value);
        self
    }

    /// Set memory_in_g_bs (unwraps Option)
    pub fn with_memory_in_g_bs(mut self, value: i64) -> Self {
        self.memory_in_g_bs = Some(value);
        self
    }

    /// Set nvmes (unwraps Option)
    pub fn with_nvmes(mut self, value: i64) -> Self {
        self.nvmes = Some(value);
        self
    }
}

impl Default for CapacityReportInstanceShapeConfig {
    fn default() -> Self {
        Self::new()
    }
}
