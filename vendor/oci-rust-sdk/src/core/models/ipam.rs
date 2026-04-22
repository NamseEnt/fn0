use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An IPAM refers to a group of VCNs, subnets, IP resources and its related properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ipam {
    /// Placeholder for description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
}

impl Ipam {
    /// Create a new Ipam
    pub fn new() -> Self {
        Self { placeholder: None }
    }

    /// Set placeholder
    pub fn set_placeholder(mut self, value: Option<String>) -> Self {
        self.placeholder = value;
        self
    }

    /// Set placeholder (unwraps Option)
    pub fn with_placeholder(mut self, value: impl Into<String>) -> Self {
        self.placeholder = Some(value.into());
        self
    }
}

impl Default for Ipam {
    fn default() -> Self {
        Self::new()
    }
}
