use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A metric emitted by managed instance resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceAnalyticSummary {
    /// The name of this metric.
    pub name: MetricName,

    /// Qualifiers provided in a metric definition. Available dimensions vary by metric namespace. Each dimension takes the form of a key-value pair. <p> Example: {@code \"managedInstanceId\": \"ocid1.managementagent.123\"}
    pub dimensions: HashMap<String, String>,

    /// The value of this metric. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub count: i64,
}

/// Required fields for ManagedInstanceAnalyticSummary
pub struct ManagedInstanceAnalyticSummaryRequired {
    /// The name of this metric.
    pub name: MetricName,

    /// Qualifiers provided in a metric definition. Available dimensions vary by metric namespace. Each dimension takes the form of a key-value pair. <p> Example: {@code \"managedInstanceId\": \"ocid1.managementagent.123\"}
    pub dimensions: HashMap<String, String>,

    /// The value of this metric. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub count: i64,
}

impl ManagedInstanceAnalyticSummary {
    /// Create a new ManagedInstanceAnalyticSummary with required fields
    pub fn new(required: ManagedInstanceAnalyticSummaryRequired) -> Self {
        Self {
            name: required.name,

            dimensions: required.dimensions,

            count: required.count,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: MetricName) -> Self {
        self.name = value;
        self
    }

    /// Set dimensions
    pub fn set_dimensions(mut self, value: HashMap<String, String>) -> Self {
        self.dimensions = value;
        self
    }

    /// Set count
    pub fn set_count(mut self, value: i64) -> Self {
        self.count = value;
        self
    }
}
