use forte_sdk::*;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The attribute every log row carries to name its project. The worker runtime
/// stamps it from a trusted value the guest cannot influence (`fn0::execute`),
/// so a query pinned to it cannot be widened by anything the caller sends.
const PROJECT_ID_ATTRIBUTE: &str = "project_id";

pub struct TelemetryClient {
    base_url: String,
    access_client_id: String,
    access_client_secret: String,
}

pub struct LogSearch<'a> {
    pub project_id: &'a str,
    pub start: &'a str,
    pub end: Option<&'a str>,
    pub stream: Option<&'a str>,
    pub contains: Option<&'a str>,
    pub regex: Option<&'a str>,
    pub limit: u32,
    pub direction: Direction,
}

pub struct LogHistogram<'a> {
    pub project_id: &'a str,
    pub start: &'a str,
    pub end: Option<&'a str>,
    pub stream: Option<&'a str>,
    pub contains: Option<&'a str>,
    pub regex: Option<&'a str>,
}

#[derive(Clone, Copy)]
pub enum Direction {
    Forward,
    Backward,
}

impl Direction {
    fn as_param(self) -> &'static str {
        match self {
            Direction::Forward => "forward",
            Direction::Backward => "backward",
        }
    }
}

#[derive(Deserialize)]
pub struct LogRow {
    pub timestamp: String,
    pub line: String,
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
}

#[derive(Deserialize)]
pub struct HistogramBucket {
    pub bucket_start: String,
    pub bucket_end: String,
    pub count: u64,
}

impl TelemetryClient {
    /// Reads the loggytracy query endpoint and its Cloudflare Access service
    /// token from the environment. These are operationally required — a control
    /// without them cannot serve any log view — so a missing one is fatal
    /// rather than silently degraded.
    pub fn from_env() -> Self {
        Self {
            base_url: std::env::var("FN0_TELEMETRY_QUERY_URL")
                .expect("FN0_TELEMETRY_QUERY_URL not set"),
            access_client_id: std::env::var("FN0_TELEMETRY_ACCESS_CLIENT_ID")
                .expect("FN0_TELEMETRY_ACCESS_CLIENT_ID not set"),
            access_client_secret: std::env::var("FN0_TELEMETRY_ACCESS_CLIENT_SECRET")
                .expect("FN0_TELEMETRY_ACCESS_CLIENT_SECRET not set"),
        }
    }

    pub async fn search_logs(&self, query: LogSearch<'_>) -> anyhow::Result<Vec<LogRow>> {
        let mut params = QueryParams::new();
        params.push("start", query.start);
        if let Some(end) = query.end {
            params.push("end", end);
        }
        params.push(
            "attr",
            &format!("{PROJECT_ID_ATTRIBUTE}={}", query.project_id),
        );
        if let Some(stream) = query.stream {
            params.push("attr", &format!("stream={stream}"));
        }
        if let Some(contains) = query.contains {
            params.push("contains", contains);
        }
        if let Some(regex) = query.regex {
            params.push("regex", regex);
        }
        params.push("limit", &query.limit.to_string());
        params.push("direction", query.direction.as_param());

        let body = self
            .get("/loggytracy/api/v1/logs", &params.encode())
            .await?;
        parse_ndjson(&body)
    }

    pub async fn histogram(
        &self,
        query: LogHistogram<'_>,
    ) -> anyhow::Result<Vec<HistogramBucket>> {
        let mut params = QueryParams::new();
        params.push("start", query.start);
        if let Some(end) = query.end {
            params.push("end", end);
        }
        params.push(
            "attr",
            &format!("{PROJECT_ID_ATTRIBUTE}={}", query.project_id),
        );
        if let Some(stream) = query.stream {
            params.push("attr", &format!("stream={stream}"));
        }
        if let Some(contains) = query.contains {
            params.push("contains", contains);
        }
        if let Some(regex) = query.regex {
            params.push("regex", regex);
        }

        let body = self
            .get("/loggytracy/api/v1/logs/histogram", &params.encode())
            .await?;
        parse_ndjson(&body)
    }

    /// loggytracy has no authentication of its own; the Cloudflare Access
    /// service token authenticates this caller at the edge, and a Transform Rule
    /// there overwrites `X-Scope-OrgID` with the tenant. So this sends the two
    /// Access headers and deliberately no tenant header of its own.
    async fn get(&self, path: &str, query: &str) -> anyhow::Result<Vec<u8>> {
        let url = format!("{}{path}?{query}", self.base_url);
        let req = http::Request::builder()
            .uri(url)
            .method("GET")
            .header("CF-Access-Client-Id", &self.access_client_id)
            .header("CF-Access-Client-Secret", &self.access_client_secret)
            .body(Vec::new())?;
        let resp = http::Client::new().send(req).await?;
        let status = resp.status().as_u16();
        let body = resp.into_body().bytes().await.to_vec();
        if !(200..300).contains(&status) {
            anyhow::bail!(
                "loggytracy {path} failed (status={status}): {}",
                String::from_utf8_lossy(&body)
            );
        }
        Ok(body)
    }
}

fn parse_ndjson<T: for<'de> Deserialize<'de>>(body: &[u8]) -> anyhow::Result<Vec<T>> {
    let text = std::str::from_utf8(body)?;
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        rows.push(serde_json::from_str(line)?);
    }
    Ok(rows)
}

struct QueryParams {
    pairs: Vec<(&'static str, String)>,
}

impl QueryParams {
    fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    fn push(&mut self, key: &'static str, value: &str) {
        self.pairs.push((key, value.to_string()));
    }

    fn encode(&self) -> String {
        self.pairs
            .iter()
            .map(|(key, value)| format!("{key}={}", percent_encode(value)))
            .collect::<Vec<_>>()
            .join("&")
    }
}

/// Percent-encodes a query-parameter value per RFC 3986: everything but the
/// unreserved set becomes `%XX`. The filter values come from user input, so an
/// unescaped `&` or `=` would otherwise forge extra parameters.
fn percent_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push(hex_digit(byte >> 4));
            encoded.push(hex_digit(byte & 0x0f));
        }
    }
    encoded
}

fn hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        _ => (b'A' + (nibble - 10)) as char,
    }
}
