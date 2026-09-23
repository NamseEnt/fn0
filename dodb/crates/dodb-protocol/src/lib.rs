use std::fmt;

use dodb_core::{
    ConditionExpectation, DocumentKey, Error, ObservedState, Revision, RevisionState,
    TransactionCondition, TransactionConflict, TransactionMutation, TransactionRequest,
};
use dodb_service::{Document, Request, Response, TransactionOutcome};

pub const MAGIC: [u8; 4] = *b"DODB";
pub const PROTOCOL_VERSION: u16 = 1;
pub const HEADER_SIZE: usize = 11;
pub const REQUEST_MESSAGE_TYPE: u8 = 1;
pub const RESPONSE_MESSAGE_TYPE: u8 = 2;
pub const MAX_STORAGE_VALUE_SIZE: usize = 64 * 1024 * 1024;
pub const MAX_STORAGE_KEY_COMPONENT_SIZE: usize = 3_990;
pub const MAX_STORAGE_ENCODED_KEY_SIZE: usize = 3_992;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtocolLimits {
    pub max_request_frame_size: usize,
    pub max_response_frame_size: usize,
    pub max_key_component_size: usize,
    pub max_encoded_key_size: usize,
    pub max_value_size: usize,
    pub max_keys: usize,
    pub max_conditions: usize,
    pub max_mutations: usize,
    pub max_query_limit: usize,
    pub max_scan_limit: usize,
    pub max_error_detail_size: usize,
}

impl Default for ProtocolLimits {
    fn default() -> Self {
        Self {
            max_request_frame_size: 68 * 1024 * 1024,
            max_response_frame_size: 68 * 1024 * 1024,
            max_key_component_size: MAX_STORAGE_KEY_COMPONENT_SIZE,
            max_encoded_key_size: MAX_STORAGE_ENCODED_KEY_SIZE,
            max_value_size: MAX_STORAGE_VALUE_SIZE,
            max_keys: 4_096,
            max_conditions: 256,
            max_mutations: 256,
            max_query_limit: 4_096,
            max_scan_limit: 4_096,
            max_error_detail_size: 4_096,
        }
    }
}

impl ProtocolLimits {
    pub fn validate(&self) -> Result<(), ProtocolError> {
        if self.max_request_frame_size < HEADER_SIZE
            || self.max_response_frame_size < HEADER_SIZE
            || self.max_key_component_size == 0
            || self.max_encoded_key_size == 0
            || self.max_value_size == 0
            || self.max_keys == 0
            || self.max_conditions == 0
            || self.max_mutations == 0
            || self.max_query_limit == 0
            || self.max_scan_limit == 0
            || self.max_error_detail_size == 0
        {
            return Err(ProtocolError::InvalidLimits);
        }
        if self.max_key_component_size > MAX_STORAGE_KEY_COMPONENT_SIZE
            || self.max_encoded_key_size > MAX_STORAGE_ENCODED_KEY_SIZE
            || self.max_value_size > MAX_STORAGE_VALUE_SIZE
        {
            return Err(ProtocolError::InvalidLimits);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameHeader {
    pub version: u16,
    pub message_type: u8,
    pub payload_length: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolError {
    InvalidLimits,
    TruncatedHeader,
    InvalidMagic,
    UnsupportedVersion(u16),
    UnknownMessageType(u8),
    TruncatedPayload,
    TrailingBytes,
    PayloadTooLarge { length: usize, maximum: usize },
    KeyTooLarge { length: usize, maximum: usize },
    LengthOverflow,
    InvalidOpcode(u8),
    InvalidStatus(u8),
    InvalidFlag(u8),
    InvalidResponseType { expected: u8, actual: u8 },
    InvalidErrorKind(u8),
    InvalidExpectation(u8),
    InvalidRevisionState(u8),
    InvalidUtf8,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLimits => write!(formatter, "protocol limits are invalid"),
            Self::TruncatedHeader => write!(formatter, "truncated frame header"),
            Self::InvalidMagic => write!(formatter, "invalid frame magic"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported protocol version {version}")
            }
            Self::UnknownMessageType(message_type) => {
                write!(formatter, "unknown message type {message_type}")
            }
            Self::TruncatedPayload => write!(formatter, "truncated frame payload"),
            Self::TrailingBytes => write!(formatter, "unexpected trailing bytes"),
            Self::PayloadTooLarge { length, maximum } => {
                write!(
                    formatter,
                    "payload length {length} exceeds maximum {maximum}"
                )
            }
            Self::KeyTooLarge { length, maximum } => {
                write!(
                    formatter,
                    "encoded key length {length} exceeds maximum {maximum}"
                )
            }
            Self::LengthOverflow => write!(formatter, "length conversion overflow"),
            Self::InvalidOpcode(opcode) => write!(formatter, "invalid operation opcode {opcode}"),
            Self::InvalidStatus(status) => write!(formatter, "invalid response status {status}"),
            Self::InvalidFlag(flag) => write!(formatter, "invalid boolean flag {flag}"),
            Self::InvalidResponseType { expected, actual } => write!(
                formatter,
                "response type {actual} does not match expected type {expected}"
            ),
            Self::InvalidErrorKind(kind) => {
                write!(formatter, "invalid application error kind {kind}")
            }
            Self::InvalidExpectation(expectation) => {
                write!(formatter, "invalid transaction expectation {expectation}")
            }
            Self::InvalidRevisionState(state) => {
                write!(formatter, "invalid revision state {state}")
            }
            Self::InvalidUtf8 => write!(formatter, "invalid UTF-8 error detail"),
        }
    }
}

impl std::error::Error for ProtocolError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationErrorKind {
    InvalidRequest,
    Overloaded,
    Conflict,
    ResponseTooLarge,
    StorageFailure,
    Corruption,
    DurabilityFailure,
    Internal,
    UnsupportedProtocol,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MutationOutcome {
    NotApplicable,
    NotApplied,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationError {
    pub kind: ApplicationErrorKind,
    pub detail: String,
    pub mutation_outcome: MutationOutcome,
    pub conflict: Option<ConflictDetails>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConflictDetails {
    pub key: DocumentKey,
    pub expected: ConditionExpectation,
    pub actual: ObservedState,
}

impl ApplicationError {
    pub fn invalid_request(detail: impl Into<String>) -> Self {
        Self {
            kind: ApplicationErrorKind::InvalidRequest,
            detail: detail.into(),
            mutation_outcome: MutationOutcome::NotApplied,
            conflict: None,
        }
    }

    pub fn from_core(error: &Error) -> Self {
        match error {
            Error::InvalidInput(detail) | Error::InvalidRequest(detail) => Self {
                kind: ApplicationErrorKind::InvalidRequest,
                detail: detail.clone(),
                mutation_outcome: MutationOutcome::NotApplied,
                conflict: None,
            },
            Error::Overloaded(detail) => Self {
                kind: ApplicationErrorKind::Overloaded,
                detail: detail.clone(),
                mutation_outcome: MutationOutcome::NotApplied,
                conflict: None,
            },
            Error::Conflict(conflict) => Self {
                kind: ApplicationErrorKind::Conflict,
                detail: "transaction condition conflict".to_owned(),
                mutation_outcome: MutationOutcome::NotApplied,
                conflict: Some(ConflictDetails::from(conflict)),
            },
            Error::ResponseTooLarge(detail) => Self {
                kind: ApplicationErrorKind::ResponseTooLarge,
                detail: detail.clone(),
                mutation_outcome: MutationOutcome::NotApplied,
                conflict: None,
            },
            Error::Corruption(detail) => Self {
                kind: ApplicationErrorKind::Corruption,
                detail: detail.clone(),
                mutation_outcome: MutationOutcome::Unknown,
                conflict: None,
            },
            Error::Io(error) => Self {
                kind: ApplicationErrorKind::StorageFailure,
                detail: error.to_string(),
                mutation_outcome: MutationOutcome::Unknown,
                conflict: None,
            },
            Error::UnsupportedFormat(detail)
            | Error::RecoveryFailure(detail)
            | Error::CheckpointFailure(detail) => Self {
                kind: ApplicationErrorKind::StorageFailure,
                detail: detail.clone(),
                mutation_outcome: MutationOutcome::Unknown,
                conflict: None,
            },
            Error::DurabilityFailure(detail) => Self {
                kind: ApplicationErrorKind::DurabilityFailure,
                detail: detail.clone(),
                mutation_outcome: MutationOutcome::Unknown,
                conflict: None,
            },
            Error::InternalInvariantViolation(detail) => Self {
                kind: ApplicationErrorKind::Internal,
                detail: detail.clone(),
                mutation_outcome: MutationOutcome::Unknown,
                conflict: None,
            },
        }
    }

    pub fn from_core_for_request(error: &Error, is_mutation: bool) -> Self {
        let mut application_error = Self::from_core(error);
        if !is_mutation {
            application_error.mutation_outcome = MutationOutcome::NotApplicable;
        }
        application_error
    }

    pub fn from_protocol(error: &ProtocolError) -> Self {
        let kind = match error {
            ProtocolError::UnsupportedVersion(_) | ProtocolError::UnknownMessageType(_) => {
                ApplicationErrorKind::UnsupportedProtocol
            }
            _ => ApplicationErrorKind::InvalidRequest,
        };
        Self {
            kind,
            detail: error.to_string(),
            mutation_outcome: MutationOutcome::NotApplied,
            conflict: None,
        }
    }
}

impl From<&TransactionConflict> for ConflictDetails {
    fn from(conflict: &TransactionConflict) -> Self {
        Self {
            key: conflict.key.clone(),
            expected: conflict.expected,
            actual: conflict.actual,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResponseEnvelope {
    Success(Response),
    Error(ApplicationError),
}

pub fn encode_request(
    tenant: dodb_core::TenantId,
    request: &Request,
    limits: ProtocolLimits,
) -> Result<Vec<u8>, ProtocolError> {
    limits.validate()?;
    let mut payload = Writer::new();
    payload.u64(tenant.get());
    encode_request_payload(&mut payload, request, limits)?;
    encode_frame(
        REQUEST_MESSAGE_TYPE,
        payload.finish(),
        limits.max_request_frame_size,
    )
}

pub fn decode_request_frame(
    frame: &[u8],
    limits: ProtocolLimits,
) -> Result<(dodb_core::TenantId, Request), ProtocolError> {
    let header = decode_header(
        frame
            .get(..HEADER_SIZE)
            .ok_or(ProtocolError::TruncatedHeader)?,
    )?;
    if frame.len() != HEADER_SIZE + header.payload_length {
        return Err(if frame.len() < HEADER_SIZE + header.payload_length {
            ProtocolError::TruncatedPayload
        } else {
            ProtocolError::TrailingBytes
        });
    }
    decode_request_parts(header, &frame[HEADER_SIZE..], limits)
}

pub fn decode_request_parts(
    header: FrameHeader,
    payload: &[u8],
    limits: ProtocolLimits,
) -> Result<(dodb_core::TenantId, Request), ProtocolError> {
    limits.validate()?;
    validate_payload_length(header, payload, limits.max_request_frame_size)?;
    if header.message_type != REQUEST_MESSAGE_TYPE {
        return Err(ProtocolError::UnknownMessageType(header.message_type));
    }
    let mut reader = Reader::new(payload);
    let tenant = dodb_core::TenantId::new(reader.u64()?);
    let request = decode_request_payload(&mut reader, limits)?;
    reader
        .finish()?
        .then_some((tenant, request))
        .ok_or(ProtocolError::TrailingBytes)
}

pub fn encode_response(
    response: &ResponseEnvelope,
    limits: ProtocolLimits,
) -> Result<Vec<u8>, ProtocolError> {
    limits.validate()?;
    let mut payload = Writer::new();
    match response {
        ResponseEnvelope::Success(response) => {
            payload.u8(0);
            payload.u8(response_opcode(response));
            encode_response_payload(&mut payload, response, limits)?;
        }
        ResponseEnvelope::Error(error) => {
            payload.u8(1);
            encode_application_error(&mut payload, error, limits)?;
        }
    }
    encode_frame(
        RESPONSE_MESSAGE_TYPE,
        payload.finish(),
        limits.max_response_frame_size,
    )
}

pub fn decode_response_frame(
    frame: &[u8],
    expected_response_type: Option<u8>,
    limits: ProtocolLimits,
) -> Result<ResponseEnvelope, ProtocolError> {
    let header = decode_header(
        frame
            .get(..HEADER_SIZE)
            .ok_or(ProtocolError::TruncatedHeader)?,
    )?;
    if frame.len() != HEADER_SIZE + header.payload_length {
        return Err(if frame.len() < HEADER_SIZE + header.payload_length {
            ProtocolError::TruncatedPayload
        } else {
            ProtocolError::TrailingBytes
        });
    }
    decode_response_parts(
        header,
        &frame[HEADER_SIZE..],
        expected_response_type,
        limits,
    )
}

pub fn decode_response_parts(
    header: FrameHeader,
    payload: &[u8],
    expected_response_type: Option<u8>,
    limits: ProtocolLimits,
) -> Result<ResponseEnvelope, ProtocolError> {
    limits.validate()?;
    validate_payload_length(header, payload, limits.max_response_frame_size)?;
    if header.message_type != RESPONSE_MESSAGE_TYPE {
        return Err(ProtocolError::UnknownMessageType(header.message_type));
    }
    let mut reader = Reader::new(payload);
    let status = reader.u8()?;
    let response = match status {
        0 => {
            let actual = reader.u8()?;
            if let Some(expected) = expected_response_type
                && actual != expected
            {
                return Err(ProtocolError::InvalidResponseType { expected, actual });
            }
            ResponseEnvelope::Success(decode_response_payload(&mut reader, actual, limits)?)
        }
        1 => ResponseEnvelope::Error(decode_application_error(&mut reader, limits)?),
        other => return Err(ProtocolError::InvalidStatus(other)),
    };
    reader
        .finish()?
        .then_some(response)
        .ok_or(ProtocolError::TrailingBytes)
}

pub fn request_opcode(request: &Request) -> u8 {
    match request {
        Request::Get { .. } => 1,
        Request::Put { .. } => 2,
        Request::Delete { .. } => 3,
        Request::Query { .. } => 4,
        Request::Scan { .. } => 5,
        Request::Transact { .. } => 8,
    }
}

pub fn response_opcode(response: &Response) -> u8 {
    match response {
        Response::Get(_) => 1,
        Response::Put(_) => 2,
        Response::Delete(_) => 3,
        Response::Query(_) => 4,
        Response::Scan(_) => 5,
        Response::Transact(_) => 8,
    }
}

pub fn decode_header(bytes: &[u8]) -> Result<FrameHeader, ProtocolError> {
    if bytes.len() < HEADER_SIZE {
        return Err(ProtocolError::TruncatedHeader);
    }
    if bytes[..4] != MAGIC {
        return Err(ProtocolError::InvalidMagic);
    }
    let version = u16::from_be_bytes([bytes[4], bytes[5]]);
    if version != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedVersion(version));
    }
    let message_type = bytes[6];
    if message_type != REQUEST_MESSAGE_TYPE && message_type != RESPONSE_MESSAGE_TYPE {
        return Err(ProtocolError::UnknownMessageType(message_type));
    }
    let payload_length = u32::from_be_bytes([bytes[7], bytes[8], bytes[9], bytes[10]]) as usize;
    Ok(FrameHeader {
        version,
        message_type,
        payload_length,
    })
}

fn encode_frame(
    message_type: u8,
    payload: Vec<u8>,
    maximum_frame_size: usize,
) -> Result<Vec<u8>, ProtocolError> {
    let frame_length = HEADER_SIZE
        .checked_add(payload.len())
        .ok_or(ProtocolError::LengthOverflow)?;
    if frame_length > maximum_frame_size {
        return Err(ProtocolError::PayloadTooLarge {
            length: frame_length,
            maximum: maximum_frame_size,
        });
    }
    let payload_length = u32::try_from(payload.len()).map_err(|_| ProtocolError::LengthOverflow)?;
    let mut frame = Vec::with_capacity(frame_length);
    frame.extend_from_slice(&MAGIC);
    frame.extend_from_slice(&PROTOCOL_VERSION.to_be_bytes());
    frame.push(message_type);
    frame.extend_from_slice(&payload_length.to_be_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

fn validate_payload_length(
    header: FrameHeader,
    payload: &[u8],
    maximum_frame_size: usize,
) -> Result<(), ProtocolError> {
    let frame_length = HEADER_SIZE
        .checked_add(header.payload_length)
        .ok_or(ProtocolError::LengthOverflow)?;
    if frame_length > maximum_frame_size {
        return Err(ProtocolError::PayloadTooLarge {
            length: frame_length,
            maximum: maximum_frame_size,
        });
    }
    if payload.len() != header.payload_length {
        return Err(if payload.len() < header.payload_length {
            ProtocolError::TruncatedPayload
        } else {
            ProtocolError::TrailingBytes
        });
    }
    Ok(())
}

fn encode_request_payload(
    writer: &mut Writer,
    request: &Request,
    limits: ProtocolLimits,
) -> Result<(), ProtocolError> {
    writer.u8(request_opcode(request));
    match request {
        Request::Get { key } => encode_key(writer, key, limits)?,
        Request::Put { key, value } => {
            encode_key(writer, key, limits)?;
            writer.bytes(value, limits.max_value_size)?;
        }
        Request::Delete { key } => encode_key(writer, key, limits)?,
        Request::Query {
            pk,
            exclusive_after_sk,
            limit,
        } => {
            validate_key_parts(
                pk.as_bytes(),
                exclusive_after_sk
                    .as_ref()
                    .map_or(&[][..], dodb_core::SortKey::as_bytes),
                limits,
            )?;
            encode_bytes(writer, pk.as_bytes(), limits.max_key_component_size)?;
            encode_optional_bytes(
                writer,
                exclusive_after_sk
                    .as_ref()
                    .map(dodb_core::SortKey::as_bytes),
                limits.max_key_component_size,
            )?;
            encode_limit(writer, *limit, limits.max_query_limit)?;
        }
        Request::Scan {
            exclusive_after_key,
            limit,
        } => {
            encode_optional_key(writer, exclusive_after_key.as_ref(), limits)?;
            encode_limit(writer, *limit, limits.max_scan_limit)?;
        }
        Request::Transact { request } => {
            encode_conditions(writer, &request.conditions, limits)?;
            encode_mutations(writer, &request.mutations, limits)?;
        }
    }
    Ok(())
}

fn decode_request_payload(
    reader: &mut Reader<'_>,
    limits: ProtocolLimits,
) -> Result<Request, ProtocolError> {
    let opcode = reader.u8()?;
    match opcode {
        1 => Ok(Request::Get {
            key: decode_key(reader, limits)?,
        }),
        2 => Ok(Request::Put {
            key: decode_key(reader, limits)?,
            value: reader.bytes(limits.max_value_size)?,
        }),
        3 => Ok(Request::Delete {
            key: decode_key(reader, limits)?,
        }),
        4 => {
            let pk_bytes = reader.bytes(limits.max_key_component_size)?;
            let exclusive_after_sk_bytes = reader.optional_bytes(limits.max_key_component_size)?;
            validate_key_parts(
                &pk_bytes,
                exclusive_after_sk_bytes.as_deref().unwrap_or(&[]),
                limits,
            )?;
            let pk = dodb_core::PrimaryKey::new(pk_bytes);
            let exclusive_after_sk = exclusive_after_sk_bytes.map(dodb_core::SortKey::new);
            let limit = reader.limit(limits.max_query_limit)?;
            Ok(Request::Query {
                pk,
                exclusive_after_sk,
                limit,
            })
        }
        5 => Ok(Request::Scan {
            exclusive_after_key: decode_optional_key(reader, limits)?,
            limit: reader.limit(limits.max_scan_limit)?,
        }),
        8 => Ok(Request::Transact {
            request: TransactionRequest::new(
                decode_conditions(reader, limits)?,
                decode_mutations(reader, limits)?,
            ),
        }),
        other => Err(ProtocolError::InvalidOpcode(other)),
    }
}

fn encode_response_payload(
    writer: &mut Writer,
    response: &Response,
    limits: ProtocolLimits,
) -> Result<(), ProtocolError> {
    match response {
        Response::Get(state) => encode_revision_state(writer, state, limits)?,
        Response::Put(revision) | Response::Delete(revision) => writer.u64(revision.get()),
        Response::Query(documents) | Response::Scan(documents) => {
            encode_documents(writer, documents, limits)?
        }
        Response::Transact(outcome) => encode_transaction_outcome(writer, *outcome),
    }
    Ok(())
}

fn decode_response_payload(
    reader: &mut Reader<'_>,
    opcode: u8,
    limits: ProtocolLimits,
) -> Result<Response, ProtocolError> {
    match opcode {
        1 => Ok(Response::Get(decode_revision_state(reader, limits)?)),
        2 => Ok(Response::Put(Revision::new(reader.u64()?))),
        3 => Ok(Response::Delete(Revision::new(reader.u64()?))),
        4 => Ok(Response::Query(decode_documents(reader, limits)?)),
        5 => Ok(Response::Scan(decode_documents(reader, limits)?)),
        8 => Ok(Response::Transact(decode_transaction_outcome(reader)?)),
        other => Err(ProtocolError::InvalidOpcode(other)),
    }
}

fn encode_transaction_outcome(writer: &mut Writer, outcome: TransactionOutcome) {
    match outcome.commit_lsn {
        Some(commit_lsn) => {
            writer.u8(1);
            writer.u64(commit_lsn.get());
        }
        None => writer.u8(0),
    }
}

fn decode_transaction_outcome(
    reader: &mut Reader<'_>,
) -> Result<TransactionOutcome, ProtocolError> {
    match reader.u8()? {
        0 => Ok(TransactionOutcome::conditions_satisfied()),
        1 => Ok(TransactionOutcome::committed(dodb_core::Lsn::new(
            reader.u64()?,
        ))),
        other => Err(ProtocolError::InvalidFlag(other)),
    }
}

fn encode_documents(
    writer: &mut Writer,
    documents: &[Document],
    limits: ProtocolLimits,
) -> Result<(), ProtocolError> {
    encode_count(writer, documents.len(), limits.max_scan_limit)?;
    for document in documents {
        encode_key(writer, &document.key, limits)?;
        writer.bytes(&document.value, limits.max_value_size)?;
        writer.u64(document.revision.get());
    }
    Ok(())
}

fn decode_documents(
    reader: &mut Reader<'_>,
    limits: ProtocolLimits,
) -> Result<Vec<Document>, ProtocolError> {
    let count = reader.count(limits.max_scan_limit)?;
    let mut documents = Vec::with_capacity(count);
    for _ in 0..count {
        documents.push(Document {
            key: decode_key(reader, limits)?,
            value: reader.bytes(limits.max_value_size)?,
            revision: Revision::new(reader.u64()?),
        });
    }
    Ok(documents)
}

fn encode_revision_state(
    writer: &mut Writer,
    state: &RevisionState,
    limits: ProtocolLimits,
) -> Result<(), ProtocolError> {
    match state {
        RevisionState::Present { value, revision } => {
            writer.u8(0);
            writer.bytes(value, limits.max_value_size)?;
            writer.u64(revision.get());
        }
        RevisionState::Missing { revision } => {
            writer.u8(1);
            writer.u64(revision.get());
        }
    }
    Ok(())
}

fn decode_revision_state(
    reader: &mut Reader<'_>,
    limits: ProtocolLimits,
) -> Result<RevisionState, ProtocolError> {
    match reader.u8()? {
        0 => Ok(RevisionState::present(
            reader.bytes(limits.max_value_size)?,
            Revision::new(reader.u64()?),
        )),
        1 => Ok(RevisionState::missing(Revision::new(reader.u64()?))),
        other => Err(ProtocolError::InvalidRevisionState(other)),
    }
}

fn encode_observed_state(writer: &mut Writer, state: ObservedState) {
    match state {
        ObservedState::Present { revision } => {
            writer.u8(0);
            writer.u64(revision.get());
        }
        ObservedState::Missing { revision } => {
            writer.u8(1);
            writer.u64(revision.get());
        }
    }
}

fn decode_observed_state(reader: &mut Reader<'_>) -> Result<ObservedState, ProtocolError> {
    match reader.u8()? {
        0 => Ok(ObservedState::present(Revision::new(reader.u64()?))),
        1 => Ok(ObservedState::missing(Revision::new(reader.u64()?))),
        other => Err(ProtocolError::InvalidRevisionState(other)),
    }
}

fn encode_conditions(
    writer: &mut Writer,
    conditions: &[TransactionCondition],
    limits: ProtocolLimits,
) -> Result<(), ProtocolError> {
    encode_count(writer, conditions.len(), limits.max_conditions)?;
    for condition in conditions {
        match condition {
            TransactionCondition::RevisionEquals {
                key,
                expected_revision,
            } => {
                writer.u8(0);
                encode_key(writer, key, limits)?;
                writer.u64(expected_revision.get());
            }
            TransactionCondition::Exists { key } => {
                writer.u8(1);
                encode_key(writer, key, limits)?;
            }
            TransactionCondition::NotExists { key } => {
                writer.u8(2);
                encode_key(writer, key, limits)?;
            }
        }
    }
    Ok(())
}

fn decode_conditions(
    reader: &mut Reader<'_>,
    limits: ProtocolLimits,
) -> Result<Vec<TransactionCondition>, ProtocolError> {
    let count = reader.count(limits.max_conditions)?;
    let mut conditions = Vec::with_capacity(count);
    for _ in 0..count {
        let kind = reader.u8()?;
        let key = decode_key(reader, limits)?;
        conditions.push(match kind {
            0 => TransactionCondition::RevisionEquals {
                key,
                expected_revision: Revision::new(reader.u64()?),
            },
            1 => TransactionCondition::Exists { key },
            2 => TransactionCondition::NotExists { key },
            other => return Err(ProtocolError::InvalidExpectation(other)),
        });
    }
    Ok(conditions)
}

fn encode_mutations(
    writer: &mut Writer,
    mutations: &[TransactionMutation],
    limits: ProtocolLimits,
) -> Result<(), ProtocolError> {
    encode_count(writer, mutations.len(), limits.max_mutations)?;
    for mutation in mutations {
        match mutation {
            TransactionMutation::Put { key, value } => {
                writer.u8(0);
                encode_key(writer, key, limits)?;
                writer.bytes(value, limits.max_value_size)?;
            }
            TransactionMutation::Delete { key } => {
                writer.u8(1);
                encode_key(writer, key, limits)?;
            }
        }
    }
    Ok(())
}

fn decode_mutations(
    reader: &mut Reader<'_>,
    limits: ProtocolLimits,
) -> Result<Vec<TransactionMutation>, ProtocolError> {
    let count = reader.count(limits.max_mutations)?;
    let mut mutations = Vec::with_capacity(count);
    for _ in 0..count {
        let kind = reader.u8()?;
        let key = decode_key(reader, limits)?;
        mutations.push(match kind {
            0 => TransactionMutation::Put {
                key,
                value: reader.bytes(limits.max_value_size)?,
            },
            1 => TransactionMutation::Delete { key },
            other => return Err(ProtocolError::InvalidOpcode(other)),
        });
    }
    Ok(mutations)
}

fn encode_key(
    writer: &mut Writer,
    key: &DocumentKey,
    limits: ProtocolLimits,
) -> Result<(), ProtocolError> {
    validate_key(key, limits)?;
    encode_bytes(writer, key.pk.as_bytes(), limits.max_key_component_size)?;
    encode_bytes(writer, key.sk.as_bytes(), limits.max_key_component_size)
}

fn decode_key(
    reader: &mut Reader<'_>,
    limits: ProtocolLimits,
) -> Result<DocumentKey, ProtocolError> {
    let key = DocumentKey::new(
        reader.bytes(limits.max_key_component_size)?,
        reader.bytes(limits.max_key_component_size)?,
    );
    validate_key(&key, limits)?;
    Ok(key)
}

fn validate_key(key: &DocumentKey, limits: ProtocolLimits) -> Result<(), ProtocolError> {
    validate_key_parts(key.pk.as_bytes(), key.sk.as_bytes(), limits)
}

fn validate_key_parts(pk: &[u8], sk: &[u8], limits: ProtocolLimits) -> Result<(), ProtocolError> {
    let key = DocumentKey::new(pk.to_vec(), sk.to_vec());
    let encoded_length = key.encoded_len();
    if encoded_length > limits.max_encoded_key_size {
        return Err(ProtocolError::KeyTooLarge {
            length: encoded_length,
            maximum: limits.max_encoded_key_size,
        });
    }
    Ok(())
}

fn encode_optional_key(
    writer: &mut Writer,
    key: Option<&DocumentKey>,
    limits: ProtocolLimits,
) -> Result<(), ProtocolError> {
    match key {
        Some(key) => {
            writer.u8(1);
            encode_key(writer, key, limits)?;
        }
        None => writer.u8(0),
    }
    Ok(())
}

fn decode_optional_key(
    reader: &mut Reader<'_>,
    limits: ProtocolLimits,
) -> Result<Option<DocumentKey>, ProtocolError> {
    match reader.flag()? {
        false => Ok(None),
        true => Ok(Some(decode_key(reader, limits)?)),
    }
}

fn encode_optional_bytes(
    writer: &mut Writer,
    bytes: Option<&[u8]>,
    maximum: usize,
) -> Result<(), ProtocolError> {
    match bytes {
        Some(bytes) => {
            writer.u8(1);
            encode_bytes(writer, bytes, maximum)?;
        }
        None => writer.u8(0),
    }
    Ok(())
}

fn encode_limit(writer: &mut Writer, limit: usize, maximum: usize) -> Result<(), ProtocolError> {
    if limit > maximum {
        return Err(ProtocolError::PayloadTooLarge {
            length: limit,
            maximum,
        });
    }
    let limit = u32::try_from(limit).map_err(|_| ProtocolError::LengthOverflow)?;
    writer.u32(limit);
    Ok(())
}

fn encode_count(writer: &mut Writer, count: usize, maximum: usize) -> Result<(), ProtocolError> {
    if count > maximum {
        return Err(ProtocolError::PayloadTooLarge {
            length: count,
            maximum,
        });
    }
    writer.u32(u32::try_from(count).map_err(|_| ProtocolError::LengthOverflow)?);
    Ok(())
}

fn encode_bytes(writer: &mut Writer, bytes: &[u8], maximum: usize) -> Result<(), ProtocolError> {
    if bytes.len() > maximum {
        return Err(ProtocolError::PayloadTooLarge {
            length: bytes.len(),
            maximum,
        });
    }
    writer.bytes_raw(bytes)
}

fn encode_application_error(
    writer: &mut Writer,
    error: &ApplicationError,
    limits: ProtocolLimits,
) -> Result<(), ProtocolError> {
    writer.u8(application_error_kind_code(error.kind));
    let detail = error
        .detail
        .chars()
        .take(limits.max_error_detail_size)
        .collect::<String>();
    encode_bytes(writer, detail.as_bytes(), limits.max_error_detail_size)?;
    writer.u8(mutation_outcome_code(error.mutation_outcome));
    match &error.conflict {
        Some(conflict) => {
            writer.u8(1);
            encode_conflict(writer, conflict, limits)?;
        }
        None => writer.u8(0),
    }
    Ok(())
}

fn decode_application_error(
    reader: &mut Reader<'_>,
    limits: ProtocolLimits,
) -> Result<ApplicationError, ProtocolError> {
    let kind = application_error_kind(reader.u8()?)?;
    let detail = String::from_utf8(reader.bytes(limits.max_error_detail_size)?)
        .map_err(|_| ProtocolError::InvalidUtf8)?;
    let mutation_outcome = mutation_outcome(reader.u8()?)?;
    let conflict = match reader.flag()? {
        true => Some(decode_conflict(reader, limits)?),
        false => None,
    };
    Ok(ApplicationError {
        kind,
        detail,
        mutation_outcome,
        conflict,
    })
}

fn encode_conflict(
    writer: &mut Writer,
    conflict: &ConflictDetails,
    limits: ProtocolLimits,
) -> Result<(), ProtocolError> {
    encode_key(writer, &conflict.key, limits)?;
    match conflict.expected {
        ConditionExpectation::RevisionEquals(revision) => {
            writer.u8(0);
            writer.u64(revision.get());
        }
        ConditionExpectation::Exists => writer.u8(1),
        ConditionExpectation::NotExists => writer.u8(2),
    }
    encode_observed_state(writer, conflict.actual);
    Ok(())
}

fn decode_conflict(
    reader: &mut Reader<'_>,
    limits: ProtocolLimits,
) -> Result<ConflictDetails, ProtocolError> {
    let key = decode_key(reader, limits)?;
    let expected = match reader.u8()? {
        0 => ConditionExpectation::RevisionEquals(Revision::new(reader.u64()?)),
        1 => ConditionExpectation::Exists,
        2 => ConditionExpectation::NotExists,
        other => return Err(ProtocolError::InvalidExpectation(other)),
    };
    Ok(ConflictDetails {
        key,
        expected,
        actual: decode_observed_state(reader)?,
    })
}

fn application_error_kind_code(kind: ApplicationErrorKind) -> u8 {
    match kind {
        ApplicationErrorKind::InvalidRequest => 0,
        ApplicationErrorKind::Overloaded => 1,
        ApplicationErrorKind::Conflict => 2,
        ApplicationErrorKind::StorageFailure => 3,
        ApplicationErrorKind::Corruption => 4,
        ApplicationErrorKind::DurabilityFailure => 5,
        ApplicationErrorKind::Internal => 6,
        ApplicationErrorKind::UnsupportedProtocol => 7,
        ApplicationErrorKind::ResponseTooLarge => 8,
    }
}

fn application_error_kind(code: u8) -> Result<ApplicationErrorKind, ProtocolError> {
    match code {
        0 => Ok(ApplicationErrorKind::InvalidRequest),
        1 => Ok(ApplicationErrorKind::Overloaded),
        2 => Ok(ApplicationErrorKind::Conflict),
        3 => Ok(ApplicationErrorKind::StorageFailure),
        4 => Ok(ApplicationErrorKind::Corruption),
        5 => Ok(ApplicationErrorKind::DurabilityFailure),
        6 => Ok(ApplicationErrorKind::Internal),
        7 => Ok(ApplicationErrorKind::UnsupportedProtocol),
        8 => Ok(ApplicationErrorKind::ResponseTooLarge),
        other => Err(ProtocolError::InvalidErrorKind(other)),
    }
}

fn mutation_outcome_code(outcome: MutationOutcome) -> u8 {
    match outcome {
        MutationOutcome::NotApplicable => 0,
        MutationOutcome::NotApplied => 1,
        MutationOutcome::Unknown => 2,
    }
}

fn mutation_outcome(code: u8) -> Result<MutationOutcome, ProtocolError> {
    match code {
        0 => Ok(MutationOutcome::NotApplicable),
        1 => Ok(MutationOutcome::NotApplied),
        2 => Ok(MutationOutcome::Unknown),
        other => Err(ProtocolError::InvalidFlag(other)),
    }
}

struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn bytes_raw(&mut self, value: &[u8]) -> Result<(), ProtocolError> {
        self.u32(u32::try_from(value.len()).map_err(|_| ProtocolError::LengthOverflow)?);
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    fn bytes(&mut self, value: &[u8], maximum: usize) -> Result<(), ProtocolError> {
        encode_bytes(self, value, maximum)
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

struct Reader<'input> {
    input: &'input [u8],
    offset: usize,
}

impl<'input> Reader<'input> {
    fn new(input: &'input [u8]) -> Self {
        Self { input, offset: 0 }
    }

    fn u8(&mut self) -> Result<u8, ProtocolError> {
        let value = *self
            .input
            .get(self.offset)
            .ok_or(ProtocolError::TruncatedPayload)?;
        self.offset += 1;
        Ok(value)
    }

    fn u32(&mut self) -> Result<u32, ProtocolError> {
        let bytes = self.take(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn u64(&mut self) -> Result<u64, ProtocolError> {
        let bytes = self.take(8)?;
        Ok(u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn bytes(&mut self, maximum: usize) -> Result<Vec<u8>, ProtocolError> {
        let length = usize::try_from(self.u32()?).map_err(|_| ProtocolError::LengthOverflow)?;
        if length > maximum {
            return Err(ProtocolError::PayloadTooLarge { length, maximum });
        }
        Ok(self.take(length)?.to_vec())
    }

    fn optional_bytes(&mut self, maximum: usize) -> Result<Option<Vec<u8>>, ProtocolError> {
        match self.flag()? {
            false => Ok(None),
            true => Ok(Some(self.bytes(maximum)?)),
        }
    }

    fn count(&mut self, maximum: usize) -> Result<usize, ProtocolError> {
        let count = usize::try_from(self.u32()?).map_err(|_| ProtocolError::LengthOverflow)?;
        if count > maximum {
            return Err(ProtocolError::PayloadTooLarge {
                length: count,
                maximum,
            });
        }
        Ok(count)
    }

    fn limit(&mut self, maximum: usize) -> Result<usize, ProtocolError> {
        self.count(maximum)
    }

    fn flag(&mut self) -> Result<bool, ProtocolError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            other => Err(ProtocolError::InvalidFlag(other)),
        }
    }

    fn take(&mut self, length: usize) -> Result<&'input [u8], ProtocolError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(ProtocolError::LengthOverflow)?;
        let bytes = self
            .input
            .get(self.offset..end)
            .ok_or(ProtocolError::TruncatedPayload)?;
        self.offset = end;
        Ok(bytes)
    }

    fn finish(&self) -> Result<bool, ProtocolError> {
        Ok(self.offset == self.input.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodb_core::{PrimaryKey, SortKey, TenantId};

    fn key(pk: &[u8], sk: &[u8]) -> DocumentKey {
        DocumentKey::new(pk.to_vec(), sk.to_vec())
    }

    fn request() -> Request {
        Request::Transact {
            request: TransactionRequest::new(
                vec![TransactionCondition::RevisionEquals {
                    key: key(&[0, 1], &[0xff]),
                    expected_revision: Revision::new(7),
                }],
                vec![TransactionMutation::Put {
                    key: key(&[0], &[1, 2]),
                    value: vec![0, 0xff, 9],
                }],
            ),
        }
    }

    #[test]
    fn every_request_round_trips() {
        let limits = ProtocolLimits::default();
        let requests = vec![
            Request::Get { key: key(&[], &[]) },
            Request::Put {
                key: key(&[0, 1], &[2]),
                value: vec![0xff, 0],
            },
            Request::Delete {
                key: key(&[1], &[]),
            },
            Request::Query {
                pk: PrimaryKey::new(vec![0, 1]),
                exclusive_after_sk: Some(SortKey::new(vec![0xff])),
                limit: 10,
            },
            Request::Scan {
                exclusive_after_key: Some(key(&[3], &[4])),
                limit: 11,
            },
            request(),
        ];
        for original in requests {
            let encoded = encode_request(TenantId::new(99), &original, limits).unwrap();
            assert_eq!(
                decode_request_frame(&encoded, limits).unwrap(),
                (TenantId::new(99), original)
            );
        }
    }

    #[test]
    fn every_response_round_trips() {
        let limits = ProtocolLimits::default();
        let responses = vec![
            Response::Get(RevisionState::missing(Revision::ZERO)),
            Response::Get(RevisionState::present(vec![0, 0xff], Revision::new(2))),
            Response::Put(Revision::new(3)),
            Response::Delete(Revision::new(4)),
            Response::Query(vec![Document {
                key: key(&[0], &[1]),
                value: vec![2, 3],
                revision: Revision::new(5),
            }]),
            Response::Scan(Vec::new()),
            Response::Transact(TransactionOutcome::conditions_satisfied()),
        ];
        for original in responses {
            let opcode = response_opcode(&original);
            let encoded =
                encode_response(&ResponseEnvelope::Success(original.clone()), limits).unwrap();
            assert_eq!(
                decode_response_frame(&encoded, Some(opcode), limits).unwrap(),
                ResponseEnvelope::Success(original)
            );
        }
    }

    #[test]
    fn conflict_and_error_round_trip() {
        let limits = ProtocolLimits::default();
        let error = ApplicationError {
            kind: ApplicationErrorKind::Conflict,
            detail: "conflict".to_owned(),
            mutation_outcome: MutationOutcome::NotApplied,
            conflict: Some(ConflictDetails {
                key: key(&[1], &[2]),
                expected: ConditionExpectation::RevisionEquals(Revision::new(8)),
                actual: ObservedState::missing(Revision::new(9)),
            }),
        };
        let encoded = encode_response(&ResponseEnvelope::Error(error.clone()), limits).unwrap();
        assert_eq!(
            decode_response_frame(&encoded, None, limits).unwrap(),
            ResponseEnvelope::Error(error)
        );
    }

    #[test]
    fn malformed_frames_are_rejected() {
        let limits = ProtocolLimits::default();
        let valid =
            encode_request(TenantId::ZERO, &Request::Get { key: key(&[], &[]) }, limits).unwrap();
        assert!(matches!(
            decode_request_frame(&valid[..HEADER_SIZE - 1], limits),
            Err(ProtocolError::TruncatedHeader)
        ));
        let mut invalid_magic = valid.clone();
        invalid_magic[0] = b'X';
        assert!(matches!(
            decode_request_frame(&invalid_magic, limits),
            Err(ProtocolError::InvalidMagic)
        ));
        let mut invalid_version = valid.clone();
        invalid_version[5] = 2;
        assert!(matches!(
            decode_request_frame(&invalid_version, limits),
            Err(ProtocolError::UnsupportedVersion(2))
        ));
        let mut invalid_type = valid.clone();
        invalid_type[6] = 99;
        assert!(matches!(
            decode_request_frame(&invalid_type, limits),
            Err(ProtocolError::UnknownMessageType(99))
        ));
        let mut absurd_length = valid[..HEADER_SIZE].to_vec();
        absurd_length[7..].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(matches!(
            decode_request_frame(&absurd_length, limits),
            Err(ProtocolError::TruncatedPayload)
        ));
        let mut invalid_opcode = valid.clone();
        invalid_opcode[HEADER_SIZE + 8] = 99;
        assert!(matches!(
            decode_request_frame(&invalid_opcode, limits),
            Err(ProtocolError::InvalidOpcode(99))
        ));
        for reserved_opcode in [6, 7] {
            let mut reserved = valid.clone();
            reserved[HEADER_SIZE + 8] = reserved_opcode;
            assert!(matches!(
                decode_request_frame(&reserved, limits),
                Err(ProtocolError::InvalidOpcode(opcode)) if opcode == reserved_opcode
            ));
        }
        let mut trailing = valid.clone();
        trailing.push(1);
        assert!(matches!(
            decode_request_frame(&trailing, limits),
            Err(ProtocolError::TrailingBytes)
        ));
    }

    #[test]
    fn response_type_mismatch_is_rejected() {
        let limits = ProtocolLimits::default();
        let encoded = encode_response(
            &ResponseEnvelope::Success(Response::Get(RevisionState::missing(Revision::ZERO))),
            limits,
        )
        .unwrap();
        assert!(matches!(
            decode_response_frame(&encoded, Some(2), limits),
            Err(ProtocolError::InvalidResponseType {
                expected: 2,
                actual: 1
            })
        ));
    }

    #[test]
    fn oversized_values_are_rejected_before_allocation() {
        let limits = ProtocolLimits {
            max_value_size: 4,
            ..ProtocolLimits::default()
        };
        let request = Request::Put {
            key: key(&[1], &[2]),
            value: vec![1, 2, 3, 4, 5],
        };
        assert!(matches!(
            encode_request(TenantId::ZERO, &request, limits),
            Err(ProtocolError::PayloadTooLarge { .. })
        ));
    }

    #[test]
    fn values_at_the_configured_limit_round_trip() {
        let limits = ProtocolLimits {
            max_value_size: 1_024,
            ..ProtocolLimits::default()
        };
        let request = Request::Put {
            key: key(&[1], &[2]),
            value: vec![0xff; 1_024],
        };
        let encoded = encode_request(TenantId::new(4), &request, limits).unwrap();
        assert_eq!(
            decode_request_frame(&encoded, limits).unwrap(),
            (TenantId::new(4), request)
        );
    }

    #[test]
    fn core_error_mapping_preserves_conflict() {
        let conflict = TransactionConflict {
            key: key(&[1], &[2]),
            expected: ConditionExpectation::Exists,
            actual: ObservedState::missing(Revision::new(10)),
        };
        let mapped = ApplicationError::from_core(&Error::conflict(conflict.clone()));
        assert_eq!(mapped.kind, ApplicationErrorKind::Conflict);
        assert_eq!(mapped.conflict, Some(ConflictDetails::from(&conflict)));
        assert_eq!(mapped.mutation_outcome, MutationOutcome::NotApplied);
        assert_eq!(
            ApplicationError::from_core(&Error::invalid_request("bad request")).mutation_outcome,
            MutationOutcome::NotApplied
        );
        assert_eq!(
            ApplicationError::from_core(&Error::overloaded("busy")).mutation_outcome,
            MutationOutcome::NotApplied
        );
    }

    #[test]
    fn canonical_key_limit_accounts_for_zero_byte_escaping() {
        let limits = ProtocolLimits::default();
        let boundary = Request::Get {
            key: key(&vec![0; 1_994], &[]),
        };
        encode_request(TenantId::ZERO, &boundary, limits).unwrap();

        let oversized = Request::Get {
            key: key(&vec![0; 1_995], &[]),
        };
        assert!(matches!(
            encode_request(TenantId::ZERO, &oversized, limits),
            Err(ProtocolError::KeyTooLarge { .. })
        ));

        let oversized_cursor = Request::Query {
            pk: PrimaryKey::new(vec![0; 1_994]),
            exclusive_after_sk: Some(SortKey::new(vec![0])),
            limit: 1,
        };
        assert!(matches!(
            encode_request(TenantId::ZERO, &oversized_cursor, limits),
            Err(ProtocolError::KeyTooLarge { .. })
        ));
    }

    #[test]
    fn application_error_round_trip_preserves_unknown_mutation_outcome() {
        let error = ApplicationError {
            kind: ApplicationErrorKind::DurabilityFailure,
            detail: "WAL sync failed".to_owned(),
            mutation_outcome: MutationOutcome::Unknown,
            conflict: None,
        };
        let encoded = encode_response(
            &ResponseEnvelope::Error(error.clone()),
            ProtocolLimits::default(),
        )
        .unwrap();
        assert_eq!(
            decode_response_frame(&encoded, None, ProtocolLimits::default()).unwrap(),
            ResponseEnvelope::Error(error)
        );
    }
}
