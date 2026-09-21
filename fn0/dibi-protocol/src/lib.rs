pub const MAGIC: [u8; 4] = *b"DIBI";
pub const PROTOCOL_VERSION: u8 = 1;
pub const FRAME_HEADER_SIZE: usize = 20;
pub const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;
pub const MAX_STRING_SIZE: usize = 1024 * 1024;
pub const MAX_DOCUMENT_SIZE: usize = 16 * 1024 * 1024;
pub const MAX_BATCH_OPS: usize = 10_000;
pub const MAX_QUERY_LIMIT: u32 = 10_000;

pub const GET_OPCODE: u8 = 0x01;
pub const PUT_OPCODE: u8 = 0x02;
pub const DELETE_OPCODE: u8 = 0x03;
pub const QUERY_OPCODE: u8 = 0x04;
pub const SCAN_OPCODE: u8 = 0x05;
pub const BATCH_OPCODE: u8 = 0x06;
pub const EXECUTE_OPS_OPCODE: u8 = 0x07;
pub const GET_WITH_VERSION_OPCODE: u8 = 0x10;
pub const BATCH_GET_WITH_VERSION_OPCODE: u8 = 0x11;
pub const TRANSACT_WRITE_ITEMS_OPCODE: u8 = 0x12;
pub const ADMIN_SCAN_OPCODE: u8 = 0x20;
pub const ADMIN_TRANSACT_WRITE_ITEMS_OPCODE: u8 = 0x21;
pub const PING_OPCODE: u8 = 0x40;
pub const STATUS_OPCODE: u8 = 0x41;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    Get = GET_OPCODE,
    Put = PUT_OPCODE,
    Delete = DELETE_OPCODE,
    Query = QUERY_OPCODE,
    Scan = SCAN_OPCODE,
    Batch = BATCH_OPCODE,
    ExecuteOps = EXECUTE_OPS_OPCODE,
    GetWithVersion = GET_WITH_VERSION_OPCODE,
    BatchGetWithVersion = BATCH_GET_WITH_VERSION_OPCODE,
    TransactWriteItems = TRANSACT_WRITE_ITEMS_OPCODE,
    AdminScan = ADMIN_SCAN_OPCODE,
    AdminTransactWriteItems = ADMIN_TRANSACT_WRITE_ITEMS_OPCODE,
    Ping = PING_OPCODE,
    Status = STATUS_OPCODE,
}

impl Opcode {
    pub fn from_u8(value: u8) -> Result<Self> {
        let opcode = match value {
            GET_OPCODE => Self::Get,
            PUT_OPCODE => Self::Put,
            DELETE_OPCODE => Self::Delete,
            QUERY_OPCODE => Self::Query,
            SCAN_OPCODE => Self::Scan,
            BATCH_OPCODE => Self::Batch,
            EXECUTE_OPS_OPCODE => Self::ExecuteOps,
            GET_WITH_VERSION_OPCODE => Self::GetWithVersion,
            BATCH_GET_WITH_VERSION_OPCODE => Self::BatchGetWithVersion,
            TRANSACT_WRITE_ITEMS_OPCODE => Self::TransactWriteItems,
            ADMIN_SCAN_OPCODE => Self::AdminScan,
            ADMIN_TRANSACT_WRITE_ITEMS_OPCODE => Self::AdminTransactWriteItems,
            PING_OPCODE => Self::Ping,
            STATUS_OPCODE => Self::Status,
            _ => return Err(ProtocolError::UnknownOpcode(value)),
        };
        Ok(opcode)
    }

    pub const fn value(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Ok = 0x00,
    InvalidRequest = 0x01,
    NotFound = 0x02,
    Conflict = 0x03,
    Unauthorized = 0x06,
    InternalError = 0x07,
}

impl Status {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0x00 => Ok(Self::Ok),
            0x01 => Ok(Self::InvalidRequest),
            0x02 => Ok(Self::NotFound),
            0x03 => Ok(Self::Conflict),
            0x06 => Ok(Self::Unauthorized),
            0x07 => Ok(Self::InternalError),
            _ => Err(ProtocolError::UnknownStatus(value)),
        }
    }

    pub const fn value(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Key {
    pub pk: String,
    pub sk: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryItem {
    pub sk: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanItem {
    pub pk: String,
    pub sk: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminItem {
    pub pk: String,
    pub sk: String,
    pub version: i64,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteOperation {
    Put {
        pk: String,
        sk: String,
        data: Vec<u8>,
    },
    Delete {
        pk: String,
        sk: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecuteOperation {
    Get {
        pk: String,
        sk: String,
    },
    Query {
        pk: String,
        after_sk: Option<String>,
        limit: u32,
    },
    Put {
        pk: String,
        sk: String,
        data: Vec<u8>,
    },
    Delete {
        pk: String,
        sk: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransactWriteOperation {
    Create {
        pk: String,
        sk: String,
        data: Vec<u8>,
    },
    Put {
        pk: String,
        sk: String,
        expected_version: i64,
        data: Vec<u8>,
    },
    Delete {
        pk: String,
        sk: String,
        expected_version: i64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequestOperation {
    Get {
        pk: String,
        sk: String,
    },
    Put {
        pk: String,
        sk: String,
        data: Vec<u8>,
    },
    Delete {
        pk: String,
        sk: String,
    },
    Query {
        pk: String,
        after_sk: Option<String>,
        limit: u32,
    },
    Scan {
        cursor: Option<Key>,
        limit: u32,
    },
    Batch(Vec<WriteOperation>),
    ExecuteOps(Vec<ExecuteOperation>),
    GetWithVersion {
        pk: String,
        sk: String,
    },
    BatchGetWithVersion(Vec<Key>),
    TransactWriteItems(Vec<TransactWriteOperation>),
    AdminScan {
        cursor: Option<Key>,
        limit: u32,
        pk_prefix: Option<String>,
    },
    AdminTransactWriteItems(Vec<TransactWriteOperation>),
    Ping,
    Status,
}

impl RequestOperation {
    pub fn opcode(&self) -> Opcode {
        match self {
            Self::Get { .. } => Opcode::Get,
            Self::Put { .. } => Opcode::Put,
            Self::Delete { .. } => Opcode::Delete,
            Self::Query { .. } => Opcode::Query,
            Self::Scan { .. } => Opcode::Scan,
            Self::Batch(_) => Opcode::Batch,
            Self::ExecuteOps(_) => Opcode::ExecuteOps,
            Self::GetWithVersion { .. } => Opcode::GetWithVersion,
            Self::BatchGetWithVersion(_) => Opcode::BatchGetWithVersion,
            Self::TransactWriteItems(_) => Opcode::TransactWriteItems,
            Self::AdminScan { .. } => Opcode::AdminScan,
            Self::AdminTransactWriteItems(_) => Opcode::AdminTransactWriteItems,
            Self::Ping => Opcode::Ping,
            Self::Status => Opcode::Status,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestFrame {
    pub version: u8,
    pub flags: u16,
    pub request_id: u64,
    pub operation: RequestOperation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecuteResult {
    Done,
    Single { found: bool, data: Option<Vec<u8>> },
    Multiple(Vec<QueryItem>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResponsePayload {
    Empty,
    ErrorMessage(String),
    Found {
        found: bool,
        data: Option<Vec<u8>>,
    },
    CommitId(u64),
    OptionalCommitId(Option<u64>),
    QueryItems(Vec<QueryItem>),
    ScanItems(Vec<ScanItem>),
    ExecuteResults(Vec<ExecuteResult>),
    Versioned {
        found: bool,
        version: Option<i64>,
        data: Option<Vec<u8>>,
    },
    VersionedItems(Vec<VersionedItem>),
    ConditionalConflicts(Vec<Conflict>),
    AdminScan {
        items: Vec<AdminItem>,
        next_cursor: Option<Key>,
    },
    Status {
        db_uuid: [u8; 16],
        last_commit_id: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionedItem {
    pub found: bool,
    pub version: Option<i64>,
    pub data: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conflict {
    pub pk: String,
    pub sk: String,
    pub expected_version: Option<i64>,
    pub actual_version: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawResponseFrame {
    pub version: u8,
    pub status: Status,
    pub flags: u16,
    pub request_id: u64,
    pub payload: Vec<u8>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("frame is truncated")]
    Truncated,
    #[error("invalid magic")]
    InvalidMagic,
    #[error("unsupported protocol version {0}")]
    UnsupportedVersion(u8),
    #[error("flags must be zero")]
    NonZeroFlags,
    #[error("unknown opcode {0:#04x}")]
    UnknownOpcode(u8),
    #[error("unknown status {0:#04x}")]
    UnknownStatus(u8),
    #[error("frame is too large: {0} bytes")]
    FrameTooLarge(usize),
    #[error("length exceeds available input")]
    LengthOutOfBounds,
    #[error("length exceeds limit {limit}: {actual}")]
    LengthLimit { limit: usize, actual: usize },
    #[error("invalid boolean value {0:#04x}")]
    InvalidBool(u8),
    #[error("invalid optional presence value {0:#04x}")]
    InvalidPresence(u8),
    #[error("invalid UTF-8 string")]
    InvalidUtf8,
    #[error("invalid payload for opcode {0:?}")]
    InvalidPayload(Opcode),
    #[error("trailing bytes after payload")]
    TrailingBytes,
    #[error("invalid response payload for status {0:?}")]
    InvalidResponsePayload(Status),
}

pub type Result<T> = std::result::Result<T, ProtocolError>;

pub fn encode_request_frame(request_id: u64, operation: &RequestOperation) -> Vec<u8> {
    let mut payload = Encoder::new();
    encode_request_payload(&mut payload, operation);
    encode_frame(operation.opcode().value(), 0, request_id, payload.finish())
}

pub fn decode_request_frame(encoded: &[u8]) -> Result<RequestFrame> {
    let (header, payload) = decode_frame_parts(encoded)?;
    let opcode = Opcode::from_u8(header.code)?;
    let operation = decode_request_payload(opcode, payload)?;
    Ok(RequestFrame {
        version: header.version,
        flags: header.flags,
        request_id: header.request_id,
        operation,
    })
}

pub fn encode_response_frame(
    request_id: u64,
    status: Status,
    payload: &ResponsePayload,
) -> Result<Vec<u8>> {
    let mut encoded_payload = Encoder::new();
    encode_response_payload(&mut encoded_payload, status, payload)?;
    Ok(encode_frame(
        status.value(),
        0,
        request_id,
        encoded_payload.finish_checked()?,
    ))
}

pub fn decode_response_frame(encoded: &[u8]) -> Result<RawResponseFrame> {
    let (header, payload) = decode_frame_parts(encoded)?;
    Ok(RawResponseFrame {
        version: header.version,
        status: Status::from_u8(header.code)?,
        flags: header.flags,
        request_id: header.request_id,
        payload: payload.to_vec(),
    })
}

pub fn decode_response_payload(
    opcode: Opcode,
    status: Status,
    encoded: &[u8],
) -> Result<ResponsePayload> {
    let mut decoder = Decoder::new(encoded);
    let payload = match status {
        Status::Ok => decode_ok_response(&mut decoder, opcode)?,
        Status::Conflict => match opcode {
            Opcode::TransactWriteItems | Opcode::AdminTransactWriteItems => {
                ResponsePayload::ConditionalConflicts(decode_conflicts(&mut decoder)?)
            }
            _ => return Err(ProtocolError::InvalidResponsePayload(status)),
        },
        Status::InvalidRequest
        | Status::NotFound
        | Status::Unauthorized
        | Status::InternalError => {
            ResponsePayload::ErrorMessage(decoder.read_string(MAX_STRING_SIZE)?)
        }
    };
    decoder.finish()?;
    Ok(payload)
}

fn decode_ok_response(decoder: &mut Decoder<'_>, opcode: Opcode) -> Result<ResponsePayload> {
    match opcode {
        Opcode::Get => decode_found(decoder),
        Opcode::Put | Opcode::Delete => Ok(ResponsePayload::CommitId(decoder.read_u64()?)),
        Opcode::Query => Ok(ResponsePayload::QueryItems(decode_query_items(decoder)?)),
        Opcode::Scan => Ok(ResponsePayload::ScanItems(decode_scan_items(decoder)?)),
        Opcode::Batch => Ok(ResponsePayload::OptionalCommitId(
            decoder.read_optional_u64()?,
        )),
        Opcode::ExecuteOps => Ok(ResponsePayload::ExecuteResults(decode_execute_results(
            decoder,
        )?)),
        Opcode::GetWithVersion => decode_versioned(decoder),
        Opcode::BatchGetWithVersion => Ok(ResponsePayload::VersionedItems(decode_versioned_items(
            decoder,
        )?)),
        Opcode::TransactWriteItems | Opcode::AdminTransactWriteItems => Ok(
            ResponsePayload::OptionalCommitId(decoder.read_optional_u64()?),
        ),
        Opcode::AdminScan => decode_admin_scan(decoder),
        Opcode::Ping => Ok(ResponsePayload::Empty),
        Opcode::Status => {
            let db_uuid = decoder.read_array_16()?;
            let last_commit_id = decoder.read_u64()?;
            Ok(ResponsePayload::Status {
                db_uuid,
                last_commit_id,
            })
        }
    }
}

fn encode_request_payload(encoder: &mut Encoder, operation: &RequestOperation) {
    match operation {
        RequestOperation::Get { pk, sk }
        | RequestOperation::Delete { pk, sk }
        | RequestOperation::GetWithVersion { pk, sk } => {
            encoder.write_string(pk);
            encoder.write_string(sk);
        }
        RequestOperation::Put { pk, sk, data } => {
            encoder.write_string(pk);
            encoder.write_string(sk);
            encoder.write_bytes(data);
        }
        RequestOperation::Query {
            pk,
            after_sk,
            limit,
        } => {
            encoder.write_string(pk);
            encoder.write_optional_string(after_sk.as_deref());
            encoder.write_u32(*limit);
        }
        RequestOperation::Scan { cursor, limit } => {
            encoder.write_optional_key(cursor.as_ref());
            encoder.write_u32(*limit);
        }
        RequestOperation::Batch(operations) => {
            encoder.write_u32(operations.len() as u32);
            for operation in operations {
                match operation {
                    WriteOperation::Put { pk, sk, data } => {
                        encoder.write_u8(0x01);
                        encoder.write_string(pk);
                        encoder.write_string(sk);
                        encoder.write_bytes(data);
                    }
                    WriteOperation::Delete { pk, sk } => {
                        encoder.write_u8(0x02);
                        encoder.write_string(pk);
                        encoder.write_string(sk);
                    }
                }
            }
        }
        RequestOperation::ExecuteOps(operations) => {
            encoder.write_u32(operations.len() as u32);
            for operation in operations {
                match operation {
                    ExecuteOperation::Get { pk, sk } => {
                        encoder.write_u8(0x01);
                        encoder.write_string(pk);
                        encoder.write_string(sk);
                    }
                    ExecuteOperation::Query {
                        pk,
                        after_sk,
                        limit,
                    } => {
                        encoder.write_u8(0x02);
                        encoder.write_string(pk);
                        encoder.write_optional_string(after_sk.as_deref());
                        encoder.write_u32(*limit);
                    }
                    ExecuteOperation::Put { pk, sk, data } => {
                        encoder.write_u8(0x03);
                        encoder.write_string(pk);
                        encoder.write_string(sk);
                        encoder.write_bytes(data);
                    }
                    ExecuteOperation::Delete { pk, sk } => {
                        encoder.write_u8(0x04);
                        encoder.write_string(pk);
                        encoder.write_string(sk);
                    }
                }
            }
        }
        RequestOperation::BatchGetWithVersion(keys) => {
            encoder.write_u32(keys.len() as u32);
            for key in keys {
                encoder.write_string(&key.pk);
                encoder.write_string(&key.sk);
            }
        }
        RequestOperation::TransactWriteItems(operations)
        | RequestOperation::AdminTransactWriteItems(operations) => {
            encode_conditional_operations(encoder, operations);
        }
        RequestOperation::AdminScan {
            cursor,
            limit,
            pk_prefix,
        } => {
            encoder.write_optional_key(cursor.as_ref());
            encoder.write_u32(*limit);
            encoder.write_optional_string(pk_prefix.as_deref());
        }
        RequestOperation::Ping | RequestOperation::Status => {}
    }
}

fn decode_request_payload(opcode: Opcode, encoded: &[u8]) -> Result<RequestOperation> {
    let mut decoder = Decoder::new(encoded);
    let operation = match opcode {
        Opcode::Get => RequestOperation::Get {
            pk: decoder.read_string(MAX_STRING_SIZE)?,
            sk: decoder.read_string(MAX_STRING_SIZE)?,
        },
        Opcode::Put => RequestOperation::Put {
            pk: decoder.read_string(MAX_STRING_SIZE)?,
            sk: decoder.read_string(MAX_STRING_SIZE)?,
            data: decoder.read_bytes(MAX_DOCUMENT_SIZE)?,
        },
        Opcode::Delete => RequestOperation::Delete {
            pk: decoder.read_string(MAX_STRING_SIZE)?,
            sk: decoder.read_string(MAX_STRING_SIZE)?,
        },
        Opcode::Query => {
            let pk = decoder.read_string(MAX_STRING_SIZE)?;
            let after_sk = decoder.read_optional_string(MAX_STRING_SIZE)?;
            let limit = decoder.read_u32()?;
            validate_limit(limit)?;
            RequestOperation::Query {
                pk,
                after_sk,
                limit,
            }
        }
        Opcode::Scan => {
            let cursor = decoder.read_optional_key()?;
            let limit = decoder.read_u32()?;
            validate_limit(limit)?;
            RequestOperation::Scan { cursor, limit }
        }
        Opcode::Batch => RequestOperation::Batch(decode_write_operations(&mut decoder)?),
        Opcode::ExecuteOps => {
            RequestOperation::ExecuteOps(decode_execute_operations(&mut decoder)?)
        }
        Opcode::GetWithVersion => RequestOperation::GetWithVersion {
            pk: decoder.read_string(MAX_STRING_SIZE)?,
            sk: decoder.read_string(MAX_STRING_SIZE)?,
        },
        Opcode::BatchGetWithVersion => {
            let count = decoder.read_count()?;
            let mut keys = Vec::with_capacity(count);
            for _ in 0..count {
                keys.push(Key {
                    pk: decoder.read_string(MAX_STRING_SIZE)?,
                    sk: decoder.read_string(MAX_STRING_SIZE)?,
                });
            }
            RequestOperation::BatchGetWithVersion(keys)
        }
        Opcode::TransactWriteItems => {
            RequestOperation::TransactWriteItems(decode_conditional_operations(&mut decoder)?)
        }
        Opcode::AdminScan => {
            let cursor = decoder.read_optional_key()?;
            let limit = decoder.read_u32()?;
            validate_limit(limit)?;
            let pk_prefix = decoder.read_optional_string(MAX_STRING_SIZE)?;
            RequestOperation::AdminScan {
                cursor,
                limit,
                pk_prefix,
            }
        }
        Opcode::AdminTransactWriteItems => {
            RequestOperation::AdminTransactWriteItems(decode_conditional_operations(&mut decoder)?)
        }
        Opcode::Ping => RequestOperation::Ping,
        Opcode::Status => RequestOperation::Status,
    };
    decoder.finish()?;
    Ok(operation)
}

fn validate_limit(limit: u32) -> Result<()> {
    if limit > MAX_QUERY_LIMIT {
        return Err(ProtocolError::LengthLimit {
            limit: MAX_QUERY_LIMIT as usize,
            actual: limit as usize,
        });
    }
    Ok(())
}

fn encode_conditional_operations(encoder: &mut Encoder, operations: &[TransactWriteOperation]) {
    encoder.write_u32(operations.len() as u32);
    for operation in operations {
        match operation {
            TransactWriteOperation::Create { pk, sk, data } => {
                encoder.write_u8(0x01);
                encoder.write_string(pk);
                encoder.write_string(sk);
                encoder.write_bytes(data);
            }
            TransactWriteOperation::Put {
                pk,
                sk,
                expected_version,
                data,
            } => {
                encoder.write_u8(0x02);
                encoder.write_string(pk);
                encoder.write_string(sk);
                encoder.write_i64(*expected_version);
                encoder.write_bytes(data);
            }
            TransactWriteOperation::Delete {
                pk,
                sk,
                expected_version,
            } => {
                encoder.write_u8(0x03);
                encoder.write_string(pk);
                encoder.write_string(sk);
                encoder.write_i64(*expected_version);
            }
        }
    }
}

fn decode_write_operations(decoder: &mut Decoder<'_>) -> Result<Vec<WriteOperation>> {
    let count = decoder.read_count()?;
    let mut operations = Vec::with_capacity(count);
    for _ in 0..count {
        let operation = match decoder.read_u8()? {
            0x01 => WriteOperation::Put {
                pk: decoder.read_string(MAX_STRING_SIZE)?,
                sk: decoder.read_string(MAX_STRING_SIZE)?,
                data: decoder.read_bytes(MAX_DOCUMENT_SIZE)?,
            },
            0x02 => WriteOperation::Delete {
                pk: decoder.read_string(MAX_STRING_SIZE)?,
                sk: decoder.read_string(MAX_STRING_SIZE)?,
            },
            _ => return Err(ProtocolError::InvalidPayload(Opcode::Batch)),
        };
        operations.push(operation);
    }
    Ok(operations)
}

fn decode_execute_operations(decoder: &mut Decoder<'_>) -> Result<Vec<ExecuteOperation>> {
    let count = decoder.read_count()?;
    let mut operations = Vec::with_capacity(count);
    for _ in 0..count {
        let operation = match decoder.read_u8()? {
            0x01 => ExecuteOperation::Get {
                pk: decoder.read_string(MAX_STRING_SIZE)?,
                sk: decoder.read_string(MAX_STRING_SIZE)?,
            },
            0x02 => {
                let pk = decoder.read_string(MAX_STRING_SIZE)?;
                let after_sk = decoder.read_optional_string(MAX_STRING_SIZE)?;
                let limit = decoder.read_u32()?;
                validate_limit(limit)?;
                ExecuteOperation::Query {
                    pk,
                    after_sk,
                    limit,
                }
            }
            0x03 => ExecuteOperation::Put {
                pk: decoder.read_string(MAX_STRING_SIZE)?,
                sk: decoder.read_string(MAX_STRING_SIZE)?,
                data: decoder.read_bytes(MAX_DOCUMENT_SIZE)?,
            },
            0x04 => ExecuteOperation::Delete {
                pk: decoder.read_string(MAX_STRING_SIZE)?,
                sk: decoder.read_string(MAX_STRING_SIZE)?,
            },
            _ => return Err(ProtocolError::InvalidPayload(Opcode::ExecuteOps)),
        };
        operations.push(operation);
    }
    Ok(operations)
}

fn decode_conditional_operations(decoder: &mut Decoder<'_>) -> Result<Vec<TransactWriteOperation>> {
    let count = decoder.read_count()?;
    let mut operations = Vec::with_capacity(count);
    for _ in 0..count {
        let operation = match decoder.read_u8()? {
            0x01 => TransactWriteOperation::Create {
                pk: decoder.read_string(MAX_STRING_SIZE)?,
                sk: decoder.read_string(MAX_STRING_SIZE)?,
                data: decoder.read_bytes(MAX_DOCUMENT_SIZE)?,
            },
            0x02 => TransactWriteOperation::Put {
                pk: decoder.read_string(MAX_STRING_SIZE)?,
                sk: decoder.read_string(MAX_STRING_SIZE)?,
                expected_version: decoder.read_i64()?,
                data: decoder.read_bytes(MAX_DOCUMENT_SIZE)?,
            },
            0x03 => TransactWriteOperation::Delete {
                pk: decoder.read_string(MAX_STRING_SIZE)?,
                sk: decoder.read_string(MAX_STRING_SIZE)?,
                expected_version: decoder.read_i64()?,
            },
            _ => {
                return Err(ProtocolError::InvalidPayload(Opcode::TransactWriteItems));
            }
        };
        operations.push(operation);
    }
    Ok(operations)
}

fn decode_found(decoder: &mut Decoder<'_>) -> Result<ResponsePayload> {
    let found = decoder.read_bool()?;
    let data = if found {
        Some(decoder.read_bytes(MAX_DOCUMENT_SIZE)?)
    } else {
        None
    };
    Ok(ResponsePayload::Found { found, data })
}

fn decode_versioned(decoder: &mut Decoder<'_>) -> Result<ResponsePayload> {
    let found = decoder.read_bool()?;
    let (version, data) = if found {
        (
            Some(decoder.read_i64()?),
            Some(decoder.read_bytes(MAX_DOCUMENT_SIZE)?),
        )
    } else {
        (None, None)
    };
    Ok(ResponsePayload::Versioned {
        found,
        version,
        data,
    })
}

fn decode_versioned_items(decoder: &mut Decoder<'_>) -> Result<Vec<VersionedItem>> {
    let count = decoder.read_count()?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let found = decoder.read_bool()?;
        let (version, data) = if found {
            (
                Some(decoder.read_i64()?),
                Some(decoder.read_bytes(MAX_DOCUMENT_SIZE)?),
            )
        } else {
            (None, None)
        };
        items.push(VersionedItem {
            found,
            version,
            data,
        });
    }
    Ok(items)
}

fn decode_query_items(decoder: &mut Decoder<'_>) -> Result<Vec<QueryItem>> {
    let count = decoder.read_count()?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        items.push(QueryItem {
            sk: decoder.read_string(MAX_STRING_SIZE)?,
            data: decoder.read_bytes(MAX_DOCUMENT_SIZE)?,
        });
    }
    Ok(items)
}

fn decode_scan_items(decoder: &mut Decoder<'_>) -> Result<Vec<ScanItem>> {
    let count = decoder.read_count()?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        items.push(ScanItem {
            pk: decoder.read_string(MAX_STRING_SIZE)?,
            sk: decoder.read_string(MAX_STRING_SIZE)?,
            data: decoder.read_bytes(MAX_DOCUMENT_SIZE)?,
        });
    }
    Ok(items)
}

fn decode_execute_results(decoder: &mut Decoder<'_>) -> Result<Vec<ExecuteResult>> {
    let count = decoder.read_count()?;
    let mut results = Vec::with_capacity(count);
    for _ in 0..count {
        results.push(match decoder.read_u8()? {
            0x01 => ExecuteResult::Done,
            0x02 => {
                let found = decoder.read_bool()?;
                let data = if found {
                    Some(decoder.read_bytes(MAX_DOCUMENT_SIZE)?)
                } else {
                    None
                };
                ExecuteResult::Single { found, data }
            }
            0x03 => ExecuteResult::Multiple(decode_query_items(decoder)?),
            _ => return Err(ProtocolError::InvalidPayload(Opcode::ExecuteOps)),
        });
    }
    Ok(results)
}

fn decode_conflicts(decoder: &mut Decoder<'_>) -> Result<Vec<Conflict>> {
    let count = decoder.read_count()?;
    let mut conflicts = Vec::with_capacity(count);
    for _ in 0..count {
        conflicts.push(Conflict {
            pk: decoder.read_string(MAX_STRING_SIZE)?,
            sk: decoder.read_string(MAX_STRING_SIZE)?,
            expected_version: decoder.read_optional_i64()?,
            actual_version: decoder.read_optional_i64()?,
        });
    }
    Ok(conflicts)
}

fn decode_admin_scan(decoder: &mut Decoder<'_>) -> Result<ResponsePayload> {
    let count = decoder.read_count()?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        items.push(AdminItem {
            pk: decoder.read_string(MAX_STRING_SIZE)?,
            sk: decoder.read_string(MAX_STRING_SIZE)?,
            version: decoder.read_i64()?,
            data: decoder.read_bytes(MAX_DOCUMENT_SIZE)?,
        });
    }
    let next_cursor = decoder.read_optional_key()?;
    Ok(ResponsePayload::AdminScan { items, next_cursor })
}

fn encode_response_payload(
    encoder: &mut Encoder,
    status: Status,
    payload: &ResponsePayload,
) -> Result<()> {
    match (status, payload) {
        (Status::Ok, ResponsePayload::Empty) => {}
        (Status::Ok, ResponsePayload::Found { found, data }) => {
            encoder.write_bool(*found);
            if *found {
                encoder.write_bytes(
                    data.as_deref()
                        .ok_or(ProtocolError::InvalidResponsePayload(status))?,
                );
            }
        }
        (Status::Ok, ResponsePayload::CommitId(commit_id)) => encoder.write_u64(*commit_id),
        (Status::Ok, ResponsePayload::OptionalCommitId(commit_id)) => {
            encoder.write_optional_u64(*commit_id)
        }
        (Status::Ok, ResponsePayload::QueryItems(items)) => encode_query_items(encoder, items),
        (Status::Ok, ResponsePayload::ScanItems(items)) => {
            encoder.write_u32(items.len() as u32);
            for item in items {
                encoder.write_string(&item.pk);
                encoder.write_string(&item.sk);
                encoder.write_bytes(&item.data);
            }
        }
        (Status::Ok, ResponsePayload::ExecuteResults(results)) => {
            encoder.write_u32(results.len() as u32);
            for result in results {
                match result {
                    ExecuteResult::Done => encoder.write_u8(0x01),
                    ExecuteResult::Single { found, data } => {
                        encoder.write_u8(0x02);
                        encoder.write_bool(*found);
                        if *found {
                            encoder.write_bytes(
                                data.as_deref()
                                    .ok_or(ProtocolError::InvalidResponsePayload(status))?,
                            );
                        }
                    }
                    ExecuteResult::Multiple(items) => {
                        encoder.write_u8(0x03);
                        encode_query_items(encoder, items);
                    }
                }
            }
        }
        (
            Status::Ok,
            ResponsePayload::Versioned {
                found,
                version,
                data,
            },
        ) => {
            encoder.write_bool(*found);
            if *found {
                encoder.write_i64(version.ok_or(ProtocolError::InvalidResponsePayload(status))?);
                encoder.write_bytes(
                    data.as_deref()
                        .ok_or(ProtocolError::InvalidResponsePayload(status))?,
                );
            }
        }
        (Status::Ok, ResponsePayload::VersionedItems(items)) => {
            encoder.write_u32(items.len() as u32);
            for item in items {
                encoder.write_bool(item.found);
                if item.found {
                    encoder.write_i64(
                        item.version
                            .ok_or(ProtocolError::InvalidResponsePayload(status))?,
                    );
                    encoder.write_bytes(
                        item.data
                            .as_deref()
                            .ok_or(ProtocolError::InvalidResponsePayload(status))?,
                    );
                }
            }
        }
        (Status::Ok, ResponsePayload::ConditionalConflicts(_)) => {
            return Err(ProtocolError::InvalidResponsePayload(status));
        }
        (Status::Conflict, ResponsePayload::ConditionalConflicts(conflicts)) => {
            encode_conflicts(encoder, conflicts);
        }
        (Status::Ok, ResponsePayload::AdminScan { items, next_cursor }) => {
            encoder.write_u32(items.len() as u32);
            for item in items {
                encoder.write_string(&item.pk);
                encoder.write_string(&item.sk);
                encoder.write_i64(item.version);
                encoder.write_bytes(&item.data);
            }
            encoder.write_optional_key(next_cursor.as_ref());
        }
        (
            Status::Ok,
            ResponsePayload::Status {
                db_uuid,
                last_commit_id,
            },
        ) => {
            encoder.write_array_16(db_uuid);
            encoder.write_u64(*last_commit_id);
        }
        (
            Status::InvalidRequest
            | Status::NotFound
            | Status::Unauthorized
            | Status::InternalError,
            ResponsePayload::ErrorMessage(message),
        ) => encoder.write_string(message),
        _ => return Err(ProtocolError::InvalidResponsePayload(status)),
    }
    Ok(())
}

fn encode_query_items(encoder: &mut Encoder, items: &[QueryItem]) {
    encoder.write_u32(items.len() as u32);
    for item in items {
        encoder.write_string(&item.sk);
        encoder.write_bytes(&item.data);
    }
}

fn encode_conflicts(encoder: &mut Encoder, conflicts: &[Conflict]) {
    encoder.write_u32(conflicts.len() as u32);
    for conflict in conflicts {
        encoder.write_string(&conflict.pk);
        encoder.write_string(&conflict.sk);
        encoder.write_optional_i64(conflict.expected_version);
        encoder.write_optional_i64(conflict.actual_version);
    }
}

fn encode_frame(code: u8, flags: u16, request_id: u64, payload: Vec<u8>) -> Vec<u8> {
    let mut frame = Vec::with_capacity(FRAME_HEADER_SIZE + payload.len());
    frame.extend_from_slice(&MAGIC);
    frame.push(PROTOCOL_VERSION);
    frame.push(code);
    frame.extend_from_slice(&flags.to_be_bytes());
    frame.extend_from_slice(&request_id.to_be_bytes());
    frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    frame.extend_from_slice(&payload);
    frame
}

struct FrameHeader {
    version: u8,
    code: u8,
    flags: u16,
    request_id: u64,
}

fn decode_frame_parts(encoded: &[u8]) -> Result<(FrameHeader, &[u8])> {
    if encoded.len() < FRAME_HEADER_SIZE {
        return Err(ProtocolError::Truncated);
    }
    if encoded.len() > MAX_FRAME_SIZE {
        return Err(ProtocolError::FrameTooLarge(encoded.len()));
    }
    if encoded[..4] != MAGIC {
        return Err(ProtocolError::InvalidMagic);
    }
    let version = encoded[4];
    if version != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedVersion(version));
    }
    let flags = u16::from_be_bytes([encoded[6], encoded[7]]);
    if flags != 0 {
        return Err(ProtocolError::NonZeroFlags);
    }
    let request_id = u64::from_be_bytes(encoded[8..16].try_into().unwrap_or_default());
    let payload_len = u32::from_be_bytes(encoded[16..20].try_into().unwrap_or_default()) as usize;
    if payload_len > MAX_FRAME_SIZE - FRAME_HEADER_SIZE {
        return Err(ProtocolError::FrameTooLarge(
            payload_len + FRAME_HEADER_SIZE,
        ));
    }
    let expected_len = FRAME_HEADER_SIZE
        .checked_add(payload_len)
        .ok_or(ProtocolError::LengthOutOfBounds)?;
    if expected_len != encoded.len() {
        return Err(ProtocolError::LengthOutOfBounds);
    }
    Ok((
        FrameHeader {
            version,
            code: encoded[5],
            flags,
            request_id,
        },
        &encoded[FRAME_HEADER_SIZE..],
    ))
}

struct Encoder {
    output: Vec<u8>,
    overflowed: bool,
}

impl Encoder {
    fn new() -> Self {
        Self {
            output: Vec::new(),
            overflowed: false,
        }
    }

    fn finish(self) -> Vec<u8> {
        self.output
    }

    fn finish_checked(self) -> Result<Vec<u8>> {
        if self.overflowed {
            Err(ProtocolError::FrameTooLarge(MAX_FRAME_SIZE + 1))
        } else {
            Ok(self.output)
        }
    }

    fn write_u8(&mut self, value: u8) {
        self.write_raw(&[value]);
    }

    fn write_u32(&mut self, value: u32) {
        self.write_raw(&value.to_be_bytes());
    }

    fn write_u64(&mut self, value: u64) {
        self.write_raw(&value.to_be_bytes());
    }

    fn write_i64(&mut self, value: i64) {
        self.write_raw(&value.to_be_bytes());
    }

    fn write_bool(&mut self, value: bool) {
        self.write_u8(u8::from(value));
    }

    fn write_array_16(&mut self, value: &[u8; 16]) {
        self.write_raw(value);
    }

    fn write_bytes(&mut self, value: &[u8]) {
        self.write_u32(value.len() as u32);
        self.write_raw(value);
    }

    fn write_string(&mut self, value: &str) {
        self.write_bytes(value.as_bytes());
    }

    fn write_optional_string(&mut self, value: Option<&str>) {
        match value {
            Some(value) => {
                self.write_u8(1);
                self.write_string(value);
            }
            None => self.write_u8(0),
        }
    }

    fn write_optional_u64(&mut self, value: Option<u64>) {
        match value {
            Some(value) => {
                self.write_u8(1);
                self.write_u64(value);
            }
            None => self.write_u8(0),
        }
    }

    fn write_optional_i64(&mut self, value: Option<i64>) {
        match value {
            Some(value) => {
                self.write_u8(1);
                self.write_i64(value);
            }
            None => self.write_u8(0),
        }
    }

    fn write_optional_key(&mut self, value: Option<&Key>) {
        match value {
            Some(key) => {
                self.write_u8(1);
                self.write_string(&key.pk);
                self.write_string(&key.sk);
            }
            None => self.write_u8(0),
        }
    }

    fn write_raw(&mut self, value: &[u8]) {
        let Some(new_length) = self.output.len().checked_add(value.len()) else {
            self.overflowed = true;
            return;
        };
        if new_length > MAX_FRAME_SIZE - FRAME_HEADER_SIZE {
            self.overflowed = true;
            return;
        }
        self.output.extend_from_slice(value);
    }
}

struct Decoder<'input> {
    input: &'input [u8],
    offset: usize,
}

impl<'input> Decoder<'input> {
    fn new(input: &'input [u8]) -> Self {
        Self { input, offset: 0 }
    }

    fn finish(&self) -> Result<()> {
        if self.offset == self.input.len() {
            Ok(())
        } else {
            Err(ProtocolError::TrailingBytes)
        }
    }

    fn read_exact(&mut self, length: usize) -> Result<&'input [u8]> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(ProtocolError::LengthOutOfBounds)?;
        let value = self
            .input
            .get(self.offset..end)
            .ok_or(ProtocolError::LengthOutOfBounds)?;
        self.offset = end;
        Ok(value)
    }

    fn read_u8(&mut self) -> Result<u8> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u32(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes(
            self.read_exact(4)?.try_into().unwrap_or_default(),
        ))
    }

    fn read_u64(&mut self) -> Result<u64> {
        Ok(u64::from_be_bytes(
            self.read_exact(8)?.try_into().unwrap_or_default(),
        ))
    }

    fn read_i64(&mut self) -> Result<i64> {
        Ok(i64::from_be_bytes(
            self.read_exact(8)?.try_into().unwrap_or_default(),
        ))
    }

    fn read_bool(&mut self) -> Result<bool> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(ProtocolError::InvalidBool(value)),
        }
    }

    fn read_array_16(&mut self) -> Result<[u8; 16]> {
        Ok(self.read_exact(16)?.try_into().unwrap_or_default())
    }

    fn read_bytes(&mut self, limit: usize) -> Result<Vec<u8>> {
        let length = self.read_u32()? as usize;
        if length > limit {
            return Err(ProtocolError::LengthLimit {
                limit,
                actual: length,
            });
        }
        Ok(self.read_exact(length)?.to_vec())
    }

    fn read_string(&mut self, limit: usize) -> Result<String> {
        let value = self.read_bytes(limit)?;
        String::from_utf8(value).map_err(|_| ProtocolError::InvalidUtf8)
    }

    fn read_optional_string(&mut self, limit: usize) -> Result<Option<String>> {
        match self.read_u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.read_string(limit)?)),
            value => Err(ProtocolError::InvalidPresence(value)),
        }
    }

    fn read_optional_u64(&mut self) -> Result<Option<u64>> {
        match self.read_u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.read_u64()?)),
            value => Err(ProtocolError::InvalidPresence(value)),
        }
    }

    fn read_optional_i64(&mut self) -> Result<Option<i64>> {
        match self.read_u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.read_i64()?)),
            value => Err(ProtocolError::InvalidPresence(value)),
        }
    }

    fn read_optional_key(&mut self) -> Result<Option<Key>> {
        match self.read_u8()? {
            0 => Ok(None),
            1 => Ok(Some(Key {
                pk: self.read_string(MAX_STRING_SIZE)?,
                sk: self.read_string(MAX_STRING_SIZE)?,
            })),
            value => Err(ProtocolError::InvalidPresence(value)),
        }
    }

    fn read_count(&mut self) -> Result<usize> {
        let count = self.read_u32()? as usize;
        if count > MAX_BATCH_OPS {
            return Err(ProtocolError::LengthLimit {
                limit: MAX_BATCH_OPS,
                actual: count,
            });
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitive_and_request_round_trip() {
        let operation = RequestOperation::Put {
            pk: "a\0한글".to_owned(),
            sk: "b".to_owned(),
            data: vec![0, 1, 255],
        };
        let encoded = encode_request_frame(42, &operation);
        let decoded = decode_request_frame(&encoded).unwrap();
        assert_eq!(decoded.request_id, 42);
        assert_eq!(decoded.operation, operation);
    }

    #[test]
    fn invalid_primitives_are_rejected() {
        let mut operation = encode_request_frame(1, &RequestOperation::Ping);
        operation.extend_from_slice(&[1]);
        assert!(matches!(
            decode_request_frame(&operation),
            Err(ProtocolError::LengthOutOfBounds)
        ));

        let payload = vec![0, 0, 0, 0, 0, 0, 0];
        let frame = encode_frame(GET_OPCODE, 0, 1, payload);
        assert!(matches!(
            decode_request_frame(&frame),
            Err(ProtocolError::LengthOutOfBounds)
        ));

        let mut invalid_bool = Decoder::new(&[2]);
        assert!(matches!(
            invalid_bool.read_bool(),
            Err(ProtocolError::InvalidBool(2))
        ));
        let invalid_utf8 = [0, 0, 0, 1, 0xff];
        let mut decoder = Decoder::new(&invalid_utf8);
        assert!(matches!(
            decoder.read_string(MAX_STRING_SIZE),
            Err(ProtocolError::InvalidUtf8)
        ));
    }

    #[test]
    fn malformed_headers_are_rejected() {
        assert!(matches!(
            decode_request_frame(&[]),
            Err(ProtocolError::Truncated)
        ));
        let mut wrong_magic = encode_request_frame(1, &RequestOperation::Ping);
        wrong_magic[0] = b'X';
        assert!(matches!(
            decode_request_frame(&wrong_magic),
            Err(ProtocolError::InvalidMagic)
        ));
        let mut wrong_version = encode_request_frame(1, &RequestOperation::Ping);
        wrong_version[4] = 2;
        assert!(matches!(
            decode_request_frame(&wrong_version),
            Err(ProtocolError::UnsupportedVersion(2))
        ));
        let mut flags = encode_request_frame(1, &RequestOperation::Ping);
        flags[6] = 1;
        assert!(matches!(
            decode_request_frame(&flags),
            Err(ProtocolError::NonZeroFlags)
        ));
        let mut trailing = encode_request_frame(1, &RequestOperation::Ping);
        trailing[19] = 1;
        assert!(matches!(
            decode_request_frame(&trailing),
            Err(ProtocolError::LengthOutOfBounds)
        ));
        let unknown_opcode = encode_frame(0xff, 0, 1, Vec::new());
        assert!(matches!(
            decode_request_frame(&unknown_opcode),
            Err(ProtocolError::UnknownOpcode(0xff))
        ));
        let mut oversized = encode_request_frame(1, &RequestOperation::Ping);
        oversized[16..20].copy_from_slice(&(MAX_FRAME_SIZE as u32).to_be_bytes());
        assert!(matches!(
            decode_request_frame(&oversized),
            Err(ProtocolError::FrameTooLarge(_))
        ));
    }

    #[test]
    fn response_payload_round_trip() {
        let encoded = encode_response_frame(
            7,
            Status::Ok,
            &ResponsePayload::Versioned {
                found: true,
                version: Some(-2),
                data: Some(vec![0, 255]),
            },
        )
        .unwrap();
        let frame = decode_response_frame(&encoded).unwrap();
        let payload =
            decode_response_payload(Opcode::GetWithVersion, frame.status, &frame.payload).unwrap();
        assert_eq!(frame.request_id, 7);
        assert_eq!(
            payload,
            ResponsePayload::Versioned {
                found: true,
                version: Some(-2),
                data: Some(vec![0, 255])
            }
        );
    }

    #[test]
    fn optimistic_transaction_operations_round_trip() {
        let operations = vec![
            RequestOperation::GetWithVersion {
                pk: "a".to_owned(),
                sk: "one".to_owned(),
            },
            RequestOperation::BatchGetWithVersion(vec![
                Key {
                    pk: "a".to_owned(),
                    sk: "one".to_owned(),
                },
                Key {
                    pk: "missing".to_owned(),
                    sk: "key".to_owned(),
                },
            ]),
            RequestOperation::TransactWriteItems(vec![
                TransactWriteOperation::Create {
                    pk: "new".to_owned(),
                    sk: "key".to_owned(),
                    data: vec![1, 2],
                },
                TransactWriteOperation::Put {
                    pk: "a".to_owned(),
                    sk: "one".to_owned(),
                    expected_version: 3,
                    data: vec![3],
                },
                TransactWriteOperation::Delete {
                    pk: "b".to_owned(),
                    sk: "two".to_owned(),
                    expected_version: 7,
                },
            ]),
        ];
        for operation in operations {
            let encoded = encode_request_frame(11, &operation);
            assert_eq!(decode_request_frame(&encoded).unwrap().operation, operation);
        }

        let encoded = encode_response_frame(
            12,
            Status::Ok,
            &ResponsePayload::VersionedItems(vec![
                VersionedItem {
                    found: true,
                    version: Some(3),
                    data: Some(vec![4]),
                },
                VersionedItem {
                    found: false,
                    version: None,
                    data: None,
                },
            ]),
        )
        .unwrap();
        let frame = decode_response_frame(&encoded).unwrap();
        assert_eq!(
            decode_response_payload(Opcode::BatchGetWithVersion, frame.status, &frame.payload)
                .unwrap(),
            ResponsePayload::VersionedItems(vec![
                VersionedItem {
                    found: true,
                    version: Some(3),
                    data: Some(vec![4]),
                },
                VersionedItem {
                    found: false,
                    version: None,
                    data: None,
                },
            ])
        );
    }
}
