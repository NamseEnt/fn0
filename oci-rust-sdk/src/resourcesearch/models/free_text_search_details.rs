use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A request containing arbitrary text that must be present in the resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeTextSearchDetails {
    /// The text to search for.
    pub text: String,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for FreeTextSearchDetails
pub struct FreeTextSearchDetailsRequired {
    /// The text to search for.
    pub text: String,

    pub r#type: String,
}

impl FreeTextSearchDetails {
    /// Create a new FreeTextSearchDetails with required fields
    pub fn new(required: FreeTextSearchDetailsRequired) -> Self {
        Self {
            text: required.text,

            r#type: required.r#type,
        }
    }

    /// Set text
    pub fn set_text(mut self, value: String) -> Self {
        self.text = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
