//! S3-style REST backend. Requests are sent unsigned to the injected
//! placeholder endpoint; the worker's object-storage hijack rewrites the host,
//! prepends the project bucket, and adds the SigV4 signature.

use crate::runtime::{self, Request, Response};
use crate::{Error, ListEntry, ObjectList, ObjectMetadata, Result};
use bytes::Bytes;
use std::time::Duration;

#[derive(Clone)]
pub(crate) struct HttpBucket {
    endpoint: String,
}

impl HttpBucket {
    pub(crate) fn new(endpoint: String) -> Self {
        Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
        }
    }

    fn object_url(&self, key: &str) -> String {
        format!("{}/{}", self.endpoint, encode_path(key))
    }

    pub(crate) async fn put(
        &self,
        key: &str,
        data: Bytes,
        content_type: Option<&str>,
    ) -> Result<()> {
        let mut headers = vec![("content-length".to_string(), data.len().to_string())];
        if let Some(ct) = content_type {
            headers.push(("content-type".to_string(), ct.to_string()));
        }
        let response = runtime::send(Request {
            method: "PUT",
            url: self.object_url(key),
            headers,
            body: Some(data),
        })
        .await?;
        match response.status {
            200..=299 => Ok(()),
            _ => Err(unexpected(&response)),
        }
    }

    pub(crate) async fn get(&self, key: &str) -> Result<Option<Bytes>> {
        let response = runtime::send(Request {
            method: "GET",
            url: self.object_url(key),
            headers: Vec::new(),
            body: None,
        })
        .await?;
        match response.status {
            200 => Ok(Some(response.body)),
            404 => Ok(None),
            _ => Err(unexpected(&response)),
        }
    }

    pub(crate) async fn head(&self, key: &str) -> Result<Option<ObjectMetadata>> {
        let response = runtime::send(Request {
            method: "HEAD",
            url: self.object_url(key),
            headers: Vec::new(),
            body: None,
        })
        .await?;
        match response.status {
            200 => Ok(Some(metadata_from_headers(&response.headers))),
            404 => Ok(None),
            _ => Err(unexpected(&response)),
        }
    }

    pub(crate) async fn delete(&self, key: &str) -> Result<()> {
        let response = runtime::send(Request {
            method: "DELETE",
            url: self.object_url(key),
            headers: Vec::new(),
            body: None,
        })
        .await?;
        match response.status {
            200..=299 | 404 => Ok(()),
            _ => Err(unexpected(&response)),
        }
    }

    pub(crate) async fn list(
        &self,
        prefix: &str,
        after: Option<&str>,
        limit: usize,
    ) -> Result<ObjectList> {
        let mut query = format!("?list-type=2&max-keys={limit}");
        if !prefix.is_empty() {
            query.push_str(&format!("&prefix={}", encode_query(prefix)));
        }
        if let Some(after) = after {
            query.push_str(&format!("&start-after={}", encode_query(after)));
        }
        let response = runtime::send(Request {
            method: "GET",
            url: format!("{}/{}", self.endpoint, query),
            headers: Vec::new(),
            body: None,
        })
        .await?;
        if response.status != 200 {
            return Err(unexpected(&response));
        }
        parse_list_xml(&response.body)
    }

    /// Asks the object-storage hijack to mint a presigned URL. The request is
    /// an ordinary `GET` to the placeholder endpoint carrying a marker header;
    /// the hijack signs the URL out of band and returns it as the body.
    pub(crate) async fn presigned_url(
        &self,
        key: &str,
        method: &str,
        expires: Duration,
        content_length: Option<u64>,
    ) -> Result<String> {
        let header = match method {
            "GET" => "x-fn0-presign-get",
            "PUT" => "x-fn0-presign-put",
            other => {
                return Err(Error::Transport(format!(
                    "unsupported presign method: {other}"
                )));
            }
        };
        let mut headers = vec![(header.to_string(), expires.as_secs().to_string())];
        if let Some(content_length) = content_length {
            headers.push((
                "x-fn0-presign-content-length".to_string(),
                content_length.to_string(),
            ));
        }
        let response = runtime::send(Request {
            method: "GET",
            url: self.object_url(key),
            headers,
            body: None,
        })
        .await?;
        if response.status != 200 {
            return Err(unexpected(&response));
        }
        String::from_utf8(response.body.to_vec()).map_err(|e| Error::Parse(e.to_string()))
    }
}

fn unexpected(response: &Response) -> Error {
    Error::UnexpectedStatus {
        status: response.status,
        message: String::from_utf8_lossy(&response.body)
            .chars()
            .take(512)
            .collect(),
    }
}

fn metadata_from_headers(headers: &[(String, String)]) -> ObjectMetadata {
    let mut metadata = ObjectMetadata {
        size: 0,
        content_type: None,
        etag: None,
    };
    for (name, value) in headers {
        match name.to_ascii_lowercase().as_str() {
            "content-length" => metadata.size = value.parse().unwrap_or(0),
            "content-type" => metadata.content_type = Some(value.clone()),
            "etag" => metadata.etag = Some(value.trim_matches('"').to_string()),
            _ => {}
        }
    }
    metadata
}

/// Percent-encodes an object key for use as a URL path, keeping `/` so callers
/// can address pseudo-directories.
fn encode_path(s: &str) -> String {
    encode(s, true)
}

/// Percent-encodes a value for use in a query string, encoding `/` as well.
fn encode_query(s: &str) -> String {
    encode(s, false)
}

fn encode(s: &str, keep_slash: bool) -> String {
    let mut out = String::with_capacity(s.len());
    for &byte in s.as_bytes() {
        let unreserved = byte.is_ascii_alphanumeric()
            || matches!(byte, b'-' | b'_' | b'.' | b'~')
            || (keep_slash && byte == b'/');
        if unreserved {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn parse_list_xml(body: &[u8]) -> Result<ObjectList> {
    let text = std::str::from_utf8(body).map_err(|e| Error::Parse(e.to_string()))?;
    let mut entries = Vec::new();
    for block in extract_blocks(text, "Contents") {
        let key = extract_tag(block, "Key")
            .ok_or_else(|| Error::Parse("<Contents> without <Key>".to_string()))?;
        let size = extract_tag(block, "Size")
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        entries.push(ListEntry {
            key: xml_unescape(key),
            size,
        });
    }
    let truncated = extract_tag(text, "IsTruncated").map(|v| v.trim() == "true") == Some(true);
    let next_cursor = if truncated {
        entries.last().map(|e| e.key.clone())
    } else {
        None
    };
    Ok(ObjectList {
        entries,
        next_cursor,
    })
}

fn extract_blocks<'a>(text: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut blocks = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find(&open) {
        let after_open = &rest[start + open.len()..];
        let Some(end) = after_open.find(&close) else {
            break;
        };
        blocks.push(&after_open[..end]);
        rest = &after_open[end + close.len()..];
    }
    blocks
}

fn extract_tag<'a>(text: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = text.find(&open)? + open.len();
    let end = text[start..].find(&close)? + start;
    Some(&text[start..end])
}

fn xml_unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[forte_sdk::test]
    async fn parses_list_response() {
        let xml = r#"<?xml version="1.0"?>
<ListBucketResult>
  <IsTruncated>true</IsTruncated>
  <Contents><Key>img/a&amp;b.png</Key><Size>12</Size></Contents>
  <Contents><Key>img/c.png</Key><Size>34</Size></Contents>
</ListBucketResult>"#;
        let list = parse_list_xml(xml.as_bytes()).unwrap();
        assert_eq!(
            list.entries,
            vec![
                ListEntry {
                    key: "img/a&b.png".to_string(),
                    size: 12
                },
                ListEntry {
                    key: "img/c.png".to_string(),
                    size: 34
                },
            ]
        );
        assert_eq!(list.next_cursor.as_deref(), Some("img/c.png"));
    }

    #[forte_sdk::test]
    async fn list_not_truncated_has_no_cursor() {
        let xml = "<ListBucketResult><IsTruncated>false</IsTruncated>\
                   <Contents><Key>a</Key><Size>1</Size></Contents></ListBucketResult>";
        let list = parse_list_xml(xml.as_bytes()).unwrap();
        assert_eq!(list.next_cursor, None);
    }

    #[forte_sdk::test]
    async fn encodes_path_keeping_slash() {
        assert_eq!(encode_path("dir/a b.png"), "dir/a%20b.png");
        assert_eq!(encode_query("dir/a"), "dir%2Fa");
    }
}
