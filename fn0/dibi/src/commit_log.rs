use prost::Message;

use crate::{DibiError, Result, decode_document_key, decode_document_value};

pub(crate) const COMMIT_RECORD_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitRecord {
    pub format_version: u32,
    pub commit_id: u64,
    pub mutations: Vec<CommitMutation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommitMutation {
    Set {
        encoded_key: Vec<u8>,
        encoded_value: Vec<u8>,
    },
    Delete {
        encoded_key: Vec<u8>,
    },
}

impl CommitRecord {
    pub(crate) fn encode(&self) -> Vec<u8> {
        ProtoCommitRecord {
            format_version: self.format_version,
            commit_id: self.commit_id,
            mutations: self
                .mutations
                .iter()
                .map(|mutation| ProtoMutation {
                    mutation: Some(match mutation {
                        CommitMutation::Set {
                            encoded_key,
                            encoded_value,
                        } => proto_mutation::Mutation::Set(ProtoSetMutation {
                            encoded_key: encoded_key.clone(),
                            encoded_value: encoded_value.clone(),
                        }),
                        CommitMutation::Delete { encoded_key } => {
                            proto_mutation::Mutation::Delete(ProtoDeleteMutation {
                                encoded_key: encoded_key.clone(),
                            })
                        }
                    }),
                })
                .collect(),
        }
        .encode_to_vec()
    }

    pub(crate) fn decode(encoded: &[u8]) -> Result<Self> {
        let record = ProtoCommitRecord::decode(encoded).map_err(|error| {
            DibiError::CorruptCommitRecord(format!("protobuf decode failed: {error}"))
        })?;
        if record.format_version != COMMIT_RECORD_FORMAT_VERSION {
            return Err(DibiError::CorruptCommitRecord(format!(
                "unsupported record format version {}",
                record.format_version
            )));
        }
        if record.commit_id == 0 {
            return Err(DibiError::CorruptCommitRecord(
                "commit id must be nonzero".to_owned(),
            ));
        }
        let mut mutations = Vec::with_capacity(record.mutations.len());
        let mut previous_key: Option<Vec<u8>> = None;
        for mutation in record.mutations {
            let mutation = match mutation.mutation.ok_or_else(|| {
                DibiError::CorruptCommitRecord("mutation payload is missing".to_owned())
            })? {
                proto_mutation::Mutation::Set(set) => {
                    validate_key_order(&previous_key, &set.encoded_key)?;
                    decode_document_key(&set.encoded_key).map_err(|error| {
                        DibiError::CorruptCommitRecord(format!("invalid Set key: {error}"))
                    })?;
                    decode_document_value(&set.encoded_value).map_err(|error| {
                        DibiError::CorruptCommitRecord(format!("invalid Set value: {error}"))
                    })?;
                    previous_key = Some(set.encoded_key.clone());
                    CommitMutation::Set {
                        encoded_key: set.encoded_key,
                        encoded_value: set.encoded_value,
                    }
                }
                proto_mutation::Mutation::Delete(delete) => {
                    validate_key_order(&previous_key, &delete.encoded_key)?;
                    decode_document_key(&delete.encoded_key).map_err(|error| {
                        DibiError::CorruptCommitRecord(format!("invalid Delete key: {error}"))
                    })?;
                    previous_key = Some(delete.encoded_key.clone());
                    CommitMutation::Delete {
                        encoded_key: delete.encoded_key,
                    }
                }
            };
            mutations.push(mutation);
        }
        Ok(Self {
            format_version: record.format_version,
            commit_id: record.commit_id,
            mutations,
        })
    }
}

fn validate_key_order(previous_key: &Option<Vec<u8>>, current_key: &[u8]) -> Result<()> {
    if previous_key
        .as_deref()
        .is_some_and(|previous| previous >= current_key)
    {
        return Err(DibiError::CorruptCommitRecord(
            "mutation keys are not strictly ascending".to_owned(),
        ));
    }
    Ok(())
}

#[derive(Clone, PartialEq, Message)]
struct ProtoCommitRecord {
    #[prost(uint32, tag = "1")]
    format_version: u32,
    #[prost(uint64, tag = "2")]
    commit_id: u64,
    #[prost(message, repeated, tag = "3")]
    mutations: Vec<ProtoMutation>,
}

#[derive(Clone, PartialEq, Message)]
struct ProtoMutation {
    #[prost(oneof = "proto_mutation::Mutation", tags = "1, 2")]
    mutation: Option<proto_mutation::Mutation>,
}

mod proto_mutation {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub(super) enum Mutation {
        #[prost(message, tag = "1")]
        Set(super::ProtoSetMutation),
        #[prost(message, tag = "2")]
        Delete(super::ProtoDeleteMutation),
    }
}

#[derive(Clone, PartialEq, Message)]
struct ProtoSetMutation {
    #[prost(bytes = "vec", tag = "1")]
    encoded_key: Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    encoded_value: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
struct ProtoDeleteMutation {
    #[prost(bytes = "vec", tag = "1")]
    encoded_key: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_record_is_an_error() {
        assert!(CommitRecord::decode(&[0xff]).is_err());
        assert!(CommitRecord::decode(&ProtoCommitRecord::default().encode_to_vec()).is_err());
    }
}
