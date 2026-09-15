//! Path and telemetry rules shared by everything that serves a `cache_static`
//! page: which request paths are cacheable, and what rendering costs.

use opentelemetry::{KeyValue, global};
use sha2::{Digest, Sha256};

pub const SLOW_GENERATION: std::time::Duration = std::time::Duration::from_secs(1);

const GENERATION_SECONDS_BUCKETS: [f64; 5] = [0.05, 0.25, 1.0, 5.0, 30.0];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticPagePathError {
    EmptyPath,
    MissingLeadingSlash,
    QueryOrFragment,
    InvalidPercentEncoding,
    DotSegment,
    Backslash,
}

impl std::fmt::Display for StaticPagePathError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPath => write!(formatter, "path is empty"),
            Self::MissingLeadingSlash => write!(formatter, "path must start with '/'"),
            Self::QueryOrFragment => {
                write!(formatter, "query strings and fragments are not allowed")
            }
            Self::InvalidPercentEncoding => {
                write!(formatter, "path contains invalid percent encoding")
            }
            Self::DotSegment => write!(formatter, "dot path segments are not allowed"),
            Self::Backslash => write!(formatter, "backslashes are not allowed"),
        }
    }
}

impl std::error::Error for StaticPagePathError {}

pub fn normalize_path(path: &str) -> Result<String, StaticPagePathError> {
    if path.is_empty() {
        return Err(StaticPagePathError::EmptyPath);
    }
    if !path.starts_with('/') {
        return Err(StaticPagePathError::MissingLeadingSlash);
    }
    if path.contains(['?', '#']) {
        return Err(StaticPagePathError::QueryOrFragment);
    }
    if path.contains('\\') {
        return Err(StaticPagePathError::Backslash);
    }

    let bytes = path.as_bytes();
    let mut normalized_bytes = Vec::with_capacity(path.len());
    let mut byte_offset = 0;
    while byte_offset < bytes.len() {
        if bytes[byte_offset] != b'%' {
            normalized_bytes.push(bytes[byte_offset]);
            byte_offset += 1;
            continue;
        }
        if byte_offset + 2 >= bytes.len() {
            return Err(StaticPagePathError::InvalidPercentEncoding);
        }
        let high =
            hex_value(bytes[byte_offset + 1]).ok_or(StaticPagePathError::InvalidPercentEncoding)?;
        let low =
            hex_value(bytes[byte_offset + 2]).ok_or(StaticPagePathError::InvalidPercentEncoding)?;
        let decoded = high * 16 + low;
        if decoded == b'\\' {
            return Err(StaticPagePathError::Backslash);
        }
        normalized_bytes.push(b'%');
        normalized_bytes.push(upper_hex_byte(high));
        normalized_bytes.push(upper_hex_byte(low));
        byte_offset += 3;
    }
    let normalized = String::from_utf8(normalized_bytes).expect("normalizing preserves UTF-8");

    for segment in normalized.split('/') {
        if is_dot_segment(segment) {
            return Err(StaticPagePathError::DotSegment);
        }
    }

    Ok(normalized)
}

pub fn path_hash_for(normalized_path: &str) -> String {
    let digest = Sha256::digest(normalized_path.as_bytes());
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(upper_hex(byte >> 4));
        encoded.push(upper_hex(byte & 0x0f));
    }
    encoded
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaticPageOutcome {
    PreflightMiss,
    Cacheable,
    Unsafe,
    Error,
}

impl StaticPageOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PreflightMiss => "preflight_miss",
            Self::Cacheable => "cacheable",
            Self::Unsafe => "unsafe",
            Self::Error => "error",
        }
    }
}

pub fn record_outcome(project_id: &str, outcome: StaticPageOutcome) {
    global::meter("fn0")
        .u64_counter("fn0.static_page.requests")
        .build()
        .add(
            1,
            &[
                KeyValue::new(
                    crate::telemetry::PROJECT_TENANT_ATTRIBUTE,
                    project_id.to_string(),
                ),
                KeyValue::new("outcome", outcome.as_str()),
            ],
        );
}

pub fn record_generation_duration(
    project_id: &str,
    outcome: StaticPageOutcome,
    duration: std::time::Duration,
) {
    global::meter("fn0")
        .f64_histogram("fn0.static_page.generation.duration")
        .with_unit("s")
        .with_boundaries(GENERATION_SECONDS_BUCKETS.to_vec())
        .build()
        .record(
            duration.as_secs_f64(),
            &[
                KeyValue::new(
                    crate::telemetry::PROJECT_TENANT_ATTRIBUTE,
                    project_id.to_string(),
                ),
                KeyValue::new("outcome", outcome.as_str()),
            ],
        );
}

fn is_dot_segment(segment: &str) -> bool {
    let mut decoded = Vec::with_capacity(segment.len());
    let bytes = segment.as_bytes();
    let mut byte_offset = 0;
    while byte_offset < bytes.len() {
        if bytes[byte_offset] == b'%' {
            let high = hex_value(bytes[byte_offset + 1]).unwrap();
            let low = hex_value(bytes[byte_offset + 2]).unwrap();
            decoded.push(high * 16 + low);
            byte_offset += 3;
        } else {
            decoded.push(bytes[byte_offset]);
            byte_offset += 1;
        }
    }
    decoded == b"." || decoded == b".."
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn upper_hex(value: u8) -> char {
    char::from(upper_hex_byte(value))
}

fn upper_hex_byte(value: u8) -> u8 {
    if value < 10 {
        b'0' + value
    } else {
        b'A' + value - 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_percent_encoding_without_merging_distinct_paths() {
        assert_eq!(normalize_path("/docs/%7euser").unwrap(), "/docs/%7Euser");
        assert_eq!(normalize_path("/docs/").unwrap(), "/docs/");
        assert_eq!(normalize_path("/docs").unwrap(), "/docs");
    }

    #[test]
    fn rejects_unsafe_or_request_specific_paths() {
        for path in [
            "",
            "about",
            "/about?preview=1",
            "/about#main",
            "/a/../b",
            "/a/%2e%2E/b",
            "/a\\b",
            "/a/%5cb",
            "/a/%",
        ] {
            assert!(normalize_path(path).is_err(), "{path}");
        }
    }

    #[test]
    fn creates_deterministic_opaque_path_hashes() {
        assert_eq!(
            path_hash_for("/"),
            "8A5EDAB282632443219E051E4ADE2D1D5BBC671C781051BF1437897CBDFEA0F1"
        );
        assert_ne!(path_hash_for("/docs"), path_hash_for("/docs/"));
    }
}
