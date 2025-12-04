use serde::{Deserialize, Serialize};

/// Multiple Compute Instance Configuration instance details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeInstanceOptions {
    /// The Compute Instance Configuration parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<super::ComputeInstanceDetails>>,
}

impl ComputeInstanceOptions {
    pub fn new() -> Self {
        Self {
            options: None,
        }
    }

    pub fn with_options(mut self, options: Vec<super::ComputeInstanceDetails>) -> Self {
        self.options = Some(options);
        self
    }
}

impl Default for ComputeInstanceOptions {
    fn default() -> Self {
        Self::new()
    }
}
