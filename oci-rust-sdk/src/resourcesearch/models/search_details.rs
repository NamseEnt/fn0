use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A base request type that contains common criteria for searching for resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchDetails {
    #[serde(rename = "type")]
    pub r#type: String,

    /// The type of matching context returned in the response. If you specify {@code HIGHLIGHTS}, then the service will highlight fragments in its response. (For more information, see ResourceSummary.searchContext and SearchContext.) The default setting is {@code NONE}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matching_context_type: Option<SearchDetailsMatchingContextType>,
}

/// Required fields for SearchDetails
pub struct SearchDetailsRequired {
    pub r#type: String,
}

impl SearchDetails {
    /// Create a new SearchDetails with required fields
    pub fn new(required: SearchDetailsRequired) -> Self {
        Self {
            r#type: required.r#type,

            matching_context_type: None,
        }
    }

    /// Set matching_context_type
    pub fn set_matching_context_type(
        mut self,
        value: Option<SearchDetailsMatchingContextType>,
    ) -> Self {
        self.matching_context_type = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set matching_context_type (unwraps Option)
    pub fn with_matching_context_type(mut self, value: SearchDetailsMatchingContextType) -> Self {
        self.matching_context_type = Some(value);
        self
    }
}
