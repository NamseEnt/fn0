use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A request that uses Search's structured query language to specify filter conditions to apply to search results. For more information about writing queries, see [Search Language Syntax](https://docs.oracle.com/iaas/Content/Search/Concepts/querysyntax.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredSearchDetails {
    /// The structured query describing which resources to search for.
    pub query: String,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for StructuredSearchDetails
pub struct StructuredSearchDetailsRequired {
    /// The structured query describing which resources to search for.
    pub query: String,

    pub r#type: String,
}

impl StructuredSearchDetails {
    /// Create a new StructuredSearchDetails with required fields
    pub fn new(required: StructuredSearchDetailsRequired) -> Self {
        Self {
            query: required.query,

            r#type: required.r#type,
        }
    }

    /// Set query
    pub fn set_query(mut self, value: String) -> Self {
        self.query = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
