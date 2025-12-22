use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Additional search context for a resource summary.
///
/// Contains information about what parts of the resource matched the search query,
/// particularly when highlights are requested.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchContext {
    /// Contains the HTML-encoded fragments of the resource that matched the search query.
    /// Keys are field names, values are arrays of matching snippets with `<h1>` tags
    /// wrapping the matched portions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlights: Option<HashMap<String, Vec<String>>>,
}

impl SearchContext {
    pub fn new() -> Self {
        Self {
            highlights: None,
        }
    }

    pub fn with_highlights(mut self, highlights: HashMap<String, Vec<String>>) -> Self {
        self.highlights = Some(highlights);
        self
    }

    /// Set highlights
    pub fn set_highlights(mut self, highlights: Option<HashMap<String, Vec<String>>>) -> Self {
        self.highlights = highlights;
        self
    }
}

impl Default for SearchContext {
    fn default() -> Self {
        Self::new()
    }
}
