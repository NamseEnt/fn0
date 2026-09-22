use base64::Engine;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

pub const PROTOCOL_VERSION: u16 = 1;
pub const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;
pub const CONTENT_TYPE: &str = "application/vnd.fn0.doc-db+json";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocDbRequest {
    pub version: u16,
    pub operation: DocDbOperation,
}

impl DocDbRequest {
    pub fn new(operation: DocDbOperation) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            operation,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum DocDbOperation {
    Get {
        key: DocDbKey,
    },
    Put {
        key: DocDbKey,
        #[serde(with = "base64_bytes")]
        data: Vec<u8>,
    },
    Delete {
        key: DocDbKey,
    },
    Query {
        pk: String,
        after_sk: Option<String>,
        limit: u64,
    },
    Scan {
        after: Option<DocDbKey>,
        limit: u64,
    },
    GetObserved {
        key: DocDbKey,
    },
    Transact {
        conditions: Vec<DocDbCondition>,
        mutations: Vec<DocDbMutation>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocDbKey {
    pub pk: String,
    pub sk: String,
}

impl DocDbKey {
    pub fn new(pk: impl Into<String>, sk: impl Into<String>) -> Self {
        Self {
            pk: pk.into(),
            sk: sk.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocDbRevision(u64);

impl DocDbRevision {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum DocDbCondition {
    RevisionEquals {
        key: DocDbKey,
        expected_revision: DocDbRevision,
    },
    Exists {
        key: DocDbKey,
    },
    NotExists {
        key: DocDbKey,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum DocDbMutation {
    Put {
        key: DocDbKey,
        #[serde(with = "base64_bytes")]
        data: Vec<u8>,
    },
    Delete {
        key: DocDbKey,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocDbResponse {
    pub version: u16,
    pub result: DocDbResult,
}

impl DocDbResponse {
    pub fn new(result: DocDbResult) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            result,
        }
    }

    pub fn error(error: DocDbError) -> Self {
        Self::new(DocDbResult::Error { error })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum DocDbResult {
    Get { data: Option<BinaryDocument> },
    Put,
    Delete,
    Query { documents: Vec<DocDbDocument> },
    Scan { documents: Vec<DocDbDocument> },
    GetObserved { document: DocDbObservedDocument },
    Transact { outcome: DocDbTransactOutcome },
    Error { error: DocDbError },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BinaryDocument {
    #[serde(with = "base64_bytes")]
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocDbDocument {
    pub key: DocDbKey,
    #[serde(with = "base64_bytes")]
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum DocDbObservedDocument {
    Present {
        #[serde(with = "base64_bytes")]
        data: Vec<u8>,
        revision: DocDbRevision,
    },
    Missing {
        revision: Option<DocDbRevision>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum DocDbTransactOutcome {
    Committed,
    Conflict { condition_index: usize },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum DocDbError {
    InvalidRequest { message: String },
    Backend { message: String },
    UnsupportedVersion { version: u16 },
}

impl fmt::Display for DocDbError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest { message } => write!(formatter, "invalid request: {message}"),
            Self::Backend { message } => write!(formatter, "backend error: {message}"),
            Self::UnsupportedVersion { version } => {
                write!(formatter, "unsupported protocol version: {version}")
            }
        }
    }
}

#[derive(Debug)]
pub enum CodecError {
    Malformed(String),
    UnsupportedVersion(u16),
    Serialize(String),
}

impl fmt::Display for CodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(message) => write!(formatter, "malformed doc-db payload: {message}"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported doc-db protocol version: {version}")
            }
            Self::Serialize(message) => {
                write!(formatter, "doc-db payload serialization failed: {message}")
            }
        }
    }
}

impl std::error::Error for CodecError {}

pub fn encode_request(request: &DocDbRequest) -> Result<Vec<u8>, CodecError> {
    serde_json::to_vec(request).map_err(|error| CodecError::Serialize(error.to_string()))
}

pub fn decode_request(bytes: &[u8]) -> Result<DocDbRequest, CodecError> {
    let request: DocDbRequest =
        serde_json::from_slice(bytes).map_err(|error| CodecError::Malformed(error.to_string()))?;
    validate_version(request.version)?;
    Ok(request)
}

pub fn encode_response(response: &DocDbResponse) -> Result<Vec<u8>, CodecError> {
    serde_json::to_vec(response).map_err(|error| CodecError::Serialize(error.to_string()))
}

pub fn decode_response(bytes: &[u8]) -> Result<DocDbResponse, CodecError> {
    let response: DocDbResponse =
        serde_json::from_slice(bytes).map_err(|error| CodecError::Malformed(error.to_string()))?;
    validate_version(response.version)?;
    Ok(response)
}

fn validate_version(version: u16) -> Result<(), CodecError> {
    if version == PROTOCOL_VERSION {
        Ok(())
    } else {
        Err(CodecError::UnsupportedVersion(version))
    }
}

mod base64_bytes {
    use super::*;

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&base64::engine::general_purpose::STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        base64::engine::general_purpose::STANDARD
            .decode(value.as_bytes())
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binary_data() -> Vec<u8> {
        let mut data = vec![0, 1, 2, 127, 128, 255];
        data.extend_from_slice(&[0; 4096]);
        data.extend_from_slice("invalid utf-8: \u{00e9}".as_bytes());
        data
    }

    #[test]
    fn round_trips_all_operations_and_binary_data() {
        let data = binary_data();
        let requests = vec![
            DocDbOperation::Get {
                key: DocDbKey::new("pk", "sk"),
            },
            DocDbOperation::Put {
                key: DocDbKey::new("pk", "put"),
                data: data.clone(),
            },
            DocDbOperation::Delete {
                key: DocDbKey::new("pk", "delete"),
            },
            DocDbOperation::Query {
                pk: "pk".to_string(),
                after_sk: Some("after".to_string()),
                limit: 17,
            },
            DocDbOperation::Scan {
                after: Some(DocDbKey::new("after-pk", "after-sk")),
                limit: 19,
            },
            DocDbOperation::GetObserved {
                key: DocDbKey::new("pk", "observed"),
            },
            DocDbOperation::Transact {
                conditions: vec![
                    DocDbCondition::RevisionEquals {
                        key: DocDbKey::new("pk", "version"),
                        expected_revision: DocDbRevision::new(4),
                    },
                    DocDbCondition::Exists {
                        key: DocDbKey::new("pk", "missing"),
                    },
                    DocDbCondition::NotExists {
                        key: DocDbKey::new("pk", "insert"),
                    },
                ],
                mutations: vec![
                    DocDbMutation::Put {
                        key: DocDbKey::new("pk", "put"),
                        data: data.clone(),
                    },
                    DocDbMutation::Delete {
                        key: DocDbKey::new("pk", "delete"),
                    },
                ],
            },
        ];

        for operation in requests {
            let request = DocDbRequest::new(operation);
            let encoded = encode_request(&request).unwrap();
            let decoded = decode_request(&encoded).unwrap();
            assert_eq!(decoded, request);
        }

        let responses = vec![
            DocDbResponse::new(DocDbResult::GetObserved {
                document: DocDbObservedDocument::Present {
                    data: data.clone(),
                    revision: DocDbRevision::new(9),
                },
            }),
            DocDbResponse::new(DocDbResult::Transact {
                outcome: DocDbTransactOutcome::Conflict { condition_index: 3 },
            }),
        ];

        for response in responses {
            let encoded = encode_response(&response).unwrap();
            let decoded = decode_response(&encoded).unwrap();
            assert_eq!(decoded, response);
        }
    }

    #[test]
    fn encodes_binary_data_as_base64_string() {
        let request = DocDbRequest::new(DocDbOperation::Put {
            key: DocDbKey::new("pk", "sk"),
            data: binary_data(),
        });
        let encoded = String::from_utf8(encode_request(&request).unwrap()).unwrap();
        assert!(encoded.contains("\"data\":\""));
        assert!(!encoded.contains("[0,1,2"));
    }

    #[test]
    fn rejects_malformed_payload() {
        assert!(matches!(
            decode_request(b"{not-json"),
            Err(CodecError::Malformed(_))
        ));
    }

    #[test]
    fn rejects_unsupported_protocol_version() {
        let request = DocDbRequest {
            version: PROTOCOL_VERSION + 1,
            operation: DocDbOperation::Get {
                key: DocDbKey::new("pk", "sk"),
            },
        };
        let encoded = encode_request(&request).unwrap();
        assert!(matches!(
            decode_request(&encoded),
            Err(CodecError::UnsupportedVersion(version)) if version == PROTOCOL_VERSION + 1
        ));
    }

    #[test]
    fn round_trips_revisions_above_i64_max() {
        let revision = DocDbRevision::new(i64::MAX as u64 + 1);
        let response = DocDbResponse::new(DocDbResult::GetObserved {
            document: DocDbObservedDocument::Missing {
                revision: Some(revision),
            },
        });
        let encoded = encode_response(&response).unwrap();
        assert_eq!(decode_response(&encoded).unwrap(), response);
    }
}
