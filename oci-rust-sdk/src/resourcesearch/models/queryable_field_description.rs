use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An individual field that can be used as part of a query filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryableFieldDescription {
    /// The type of the field, which dictates what semantics and query constraints you can use when searching or querying.
    pub field_type: QueryableFieldDescriptionFieldType,

    /// The name of the field to use when constructing the query. Field names are present for all types except {@code OBJECT}.
    pub field_name: String,

    /// Indicates that this field is actually an array of the specified field type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_array: Option<bool>,

    /// If the field type is {@code OBJECT}, then this property will provide all the individual properties of the object that can be queried.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_properties: Option<Vec<QueryableFieldDescription>>,
}

/// Required fields for QueryableFieldDescription
pub struct QueryableFieldDescriptionRequired {
    /// The type of the field, which dictates what semantics and query constraints you can use when searching or querying.
    pub field_type: QueryableFieldDescriptionFieldType,

    /// The name of the field to use when constructing the query. Field names are present for all types except {@code OBJECT}.
    pub field_name: String,
}

impl QueryableFieldDescription {
    /// Create a new QueryableFieldDescription with required fields
    pub fn new(required: QueryableFieldDescriptionRequired) -> Self {
        Self {
            field_type: required.field_type,

            field_name: required.field_name,

            is_array: None,

            object_properties: None,
        }
    }

    /// Set field_type
    pub fn set_field_type(mut self, value: QueryableFieldDescriptionFieldType) -> Self {
        self.field_type = value;
        self
    }

    /// Set field_name
    pub fn set_field_name(mut self, value: String) -> Self {
        self.field_name = value;
        self
    }

    /// Set is_array
    pub fn set_is_array(mut self, value: Option<bool>) -> Self {
        self.is_array = value;
        self
    }

    /// Set object_properties
    pub fn set_object_properties(mut self, value: Option<Vec<QueryableFieldDescription>>) -> Self {
        self.object_properties = value;
        self
    }

    /// Set is_array (unwraps Option)
    pub fn with_is_array(mut self, value: bool) -> Self {
        self.is_array = Some(value);
        self
    }

    /// Set object_properties (unwraps Option)
    pub fn with_object_properties(mut self, value: Vec<QueryableFieldDescription>) -> Self {
        self.object_properties = Some(value);
        self
    }
}
