use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A summary of the routes advertised to and received from the on-premises network.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelRouteSummary {
    /// The BGP network layer reachability information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// The age of the route. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<i64>,

    /// Indicates this is the best route.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_best_path: Option<bool>,

    /// A list of ASNs in AS_Path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_path: Option<Vec<i64>>,

    /// The source of the route advertisement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advertiser: Option<TunnelRouteSummaryAdvertiser>,
}

impl TunnelRouteSummary {
    /// Create a new TunnelRouteSummary
    pub fn new() -> Self {
        Self {
            prefix: None,

            age: None,

            is_best_path: None,

            as_path: None,

            advertiser: None,
        }
    }

    /// Set prefix
    pub fn set_prefix(mut self, value: Option<String>) -> Self {
        self.prefix = value;
        self
    }

    /// Set age
    pub fn set_age(mut self, value: Option<i64>) -> Self {
        self.age = value;
        self
    }

    /// Set is_best_path
    pub fn set_is_best_path(mut self, value: Option<bool>) -> Self {
        self.is_best_path = value;
        self
    }

    /// Set as_path
    pub fn set_as_path(mut self, value: Option<Vec<i64>>) -> Self {
        self.as_path = value;
        self
    }

    /// Set advertiser
    pub fn set_advertiser(mut self, value: Option<TunnelRouteSummaryAdvertiser>) -> Self {
        self.advertiser = value;
        self
    }

    /// Set prefix (unwraps Option)
    pub fn with_prefix(mut self, value: impl Into<String>) -> Self {
        self.prefix = Some(value.into());
        self
    }

    /// Set age (unwraps Option)
    pub fn with_age(mut self, value: i64) -> Self {
        self.age = Some(value);
        self
    }

    /// Set is_best_path (unwraps Option)
    pub fn with_is_best_path(mut self, value: bool) -> Self {
        self.is_best_path = Some(value);
        self
    }

    /// Set as_path (unwraps Option)
    pub fn with_as_path(mut self, value: Vec<i64>) -> Self {
        self.as_path = Some(value);
        self
    }

    /// Set advertiser (unwraps Option)
    pub fn with_advertiser(mut self, value: TunnelRouteSummaryAdvertiser) -> Self {
        self.advertiser = Some(value);
        self
    }
}

impl Default for TunnelRouteSummary {
    fn default() -> Self {
        Self::new()
    }
}
