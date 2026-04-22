use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryableFieldDescriptionFieldType {
    #[serde(rename = "IDENTIFIER")]
    Identifier,

    #[serde(rename = "STRING")]
    String,

    #[serde(rename = "INTEGER")]
    Integer,

    #[serde(rename = "RATIONAL")]
    Rational,

    #[serde(rename = "BOOLEAN")]
    Boolean,

    #[serde(rename = "DATETIME")]
    Datetime,

    #[serde(rename = "IP")]
    Ip,

    #[serde(rename = "OBJECT")]
    Object,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
