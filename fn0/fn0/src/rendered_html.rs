//! Path and telemetry rules shared by everything that caches server-rendered
//! HTML: which request paths are cacheable, and what rendering costs.

use opentelemetry::{KeyValue, global};
use sha2::{Digest, Sha256};

pub const PAGE_DIRECTORY: &str = "__forte/pages";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderedHtmlPathError {
    EmptyPath,
    MissingLeadingSlash,
    QueryOrFragment,
    InvalidPercentEncoding,
    DotSegment,
    Backslash,
}

impl std::fmt::Display for RenderedHtmlPathError {
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

impl std::error::Error for RenderedHtmlPathError {}

pub fn normalize_path(path: &str) -> Result<String, RenderedHtmlPathError> {
    if path.is_empty() {
        return Err(RenderedHtmlPathError::EmptyPath);
    }
    if !path.starts_with('/') {
        return Err(RenderedHtmlPathError::MissingLeadingSlash);
    }
    if path.contains(['?', '#']) {
        return Err(RenderedHtmlPathError::QueryOrFragment);
    }
    if path.contains('\\') {
        return Err(RenderedHtmlPathError::Backslash);
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
            return Err(RenderedHtmlPathError::InvalidPercentEncoding);
        }
        let high = hex_value(bytes[byte_offset + 1])
            .ok_or(RenderedHtmlPathError::InvalidPercentEncoding)?;
        let low = hex_value(bytes[byte_offset + 2])
            .ok_or(RenderedHtmlPathError::InvalidPercentEncoding)?;
        let decoded = high * 16 + low;
        if decoded == b'\\' {
            return Err(RenderedHtmlPathError::Backslash);
        }
        normalized_bytes.push(b'%');
        normalized_bytes.push(upper_hex_byte(high));
        normalized_bytes.push(upper_hex_byte(low));
        byte_offset += 3;
    }
    let normalized = String::from_utf8(normalized_bytes).expect("normalizing preserves UTF-8");

    for segment in normalized.split('/') {
        if is_dot_segment(segment) {
            return Err(RenderedHtmlPathError::DotSegment);
        }
    }

    Ok(normalized)
}

pub fn object_path_for(normalized_path: &str) -> String {
    let digest = Sha256::digest(normalized_path.as_bytes());
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(upper_hex(byte >> 4));
        encoded.push(upper_hex(byte & 0x0f));
    }
    format!("{PAGE_DIRECTORY}/{encoded}.html")
}

pub fn record_result(project_id: &str, code_version: u64, object_path: &str, result: &'static str) {
    global::meter("fn0")
        .u64_counter("fn0.rendered_html_requests")
        .build()
        .add(
            1,
            &[
                KeyValue::new("project_id", project_id.to_string()),
                KeyValue::new("code_version", code_version.to_string()),
                KeyValue::new("path_identifier", object_path.to_string()),
                KeyValue::new("result", result),
            ],
        );
}

pub fn record_generation_duration(
    project_id: &str,
    code_version: u64,
    object_path: &str,
    duration: std::time::Duration,
) {
    global::meter("fn0")
        .f64_histogram("fn0.rendered_html_generation_duration_seconds")
        .build()
        .record(
            duration.as_secs_f64(),
            &[
                KeyValue::new("project_id", project_id.to_string()),
                KeyValue::new("code_version", code_version.to_string()),
                KeyValue::new("path_identifier", object_path.to_string()),
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
    fn creates_deterministic_opaque_object_paths() {
        assert_eq!(
            object_path_for("/"),
            "__forte/pages/8A5EDAB282632443219E051E4ADE2D1D5BBC671C781051BF1437897CBDFEA0F1.html"
        );
        assert_ne!(object_path_for("/docs"), object_path_for("/docs/"));
    }
}
