use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Contains search context, such as highlighting, for found resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchContext {
    /// Describes what in each field matched the search criteria by showing highlighted values, but only for free text searches or for structured queries that use a MATCHING clause. The list of strings represents fragments of values that matched the query conditions. Highlighted values are wrapped with &lt;h1&gt;..&lt;/h1&gt; tags. All values are HTML-encoded (except &lt;h1&gt; tags).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlights: Option<HashMap<String, Vec<String>>>,
}

impl SearchContext {
    /// Create a new SearchContext
    pub fn new() -> Self {
        Self { highlights: None }
    }

    /// Set highlights
    pub fn set_highlights(mut self, value: Option<HashMap<String, Vec<String>>>) -> Self {
        self.highlights = value;
        self
    }

    /// Set highlights (unwraps Option)
    pub fn with_highlights(mut self, value: HashMap<String, Vec<String>>) -> Self {
        self.highlights = Some(value);
        self
    }
}

impl Default for SearchContext {
    fn default() -> Self {
        Self::new()
    }
}
