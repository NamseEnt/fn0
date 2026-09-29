use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use std::{fmt, fmt::Formatter};

use http::header::HeaderValue;

use crate::queue::QueueLimits;
use crate::receive::{DEFAULT_LISTEN_ADDR, DEFAULT_MAX_INFLIGHT_BYTES, DEFAULT_MAX_REQUEST_BYTES};
use crate::send::SenderConfig;

#[derive(Clone)]
pub struct Config {
    pub listen_addr: SocketAddr,
    pub data_dir: PathBuf,
    pub signy_url: String,
    pub signy_access_client_id: Option<String>,
    pub signy_access_client_secret: Option<String>,
    pub max_request_bytes: usize,
    pub max_inflight_bytes: usize,
    pub queue: QueueLimits,
    pub sender: SenderConfig,
    pub send_timeout: Duration,
    pub report_interval: Duration,
    pub zstd_level: i32,
    pub log_json: bool,
    pub host_metrics_interval: Option<Duration>,
    pub host_metrics_root: PathBuf,
    pub journal: Option<JournalConfig>,
    /// Which tenant collecty-generated telemetry belongs to.
    ///
    /// Everything collecty forwards carries its own tenant in the payload,
    /// which collecty never decodes. Collecty-generated metrics and host
    /// sources use this value. Unset disables generated exports.
    pub generated_telemetry_tenant: Option<String>,
}

impl fmt::Debug for Config {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Config")
            .field("listen_addr", &self.listen_addr)
            .field("data_dir", &self.data_dir)
            .field("signy_url", &self.signy_url)
            .field(
                "signy_access_client_id",
                &self.signy_access_client_id.as_ref().map(|_| "[REDACTED]"),
            )
            .field(
                "signy_access_client_secret",
                &self
                    .signy_access_client_secret
                    .as_ref()
                    .map(|_| "[REDACTED]"),
            )
            .field("max_request_bytes", &self.max_request_bytes)
            .field("max_inflight_bytes", &self.max_inflight_bytes)
            .field("queue", &self.queue)
            .field("sender", &self.sender)
            .field("send_timeout", &self.send_timeout)
            .field("report_interval", &self.report_interval)
            .field("zstd_level", &self.zstd_level)
            .field("log_json", &self.log_json)
            .field("host_metrics_interval", &self.host_metrics_interval)
            .field("host_metrics_root", &self.host_metrics_root)
            .field("journal", &self.journal)
            .field(
                "generated_telemetry_tenant",
                &self.generated_telemetry_tenant,
            )
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct JournalConfig {
    pub directory: Option<PathBuf>,
    pub journalctl_path: PathBuf,
    pub units: Vec<String>,
    pub min_priority: u8,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            listen_addr: DEFAULT_LISTEN_ADDR
                .parse()
                .expect("the default listen address parses"),
            data_dir: PathBuf::from("/var/lib/collecty"),
            signy_url: "http://127.0.0.1:3100".to_string(),
            signy_access_client_id: None,
            signy_access_client_secret: None,
            max_request_bytes: DEFAULT_MAX_REQUEST_BYTES,
            max_inflight_bytes: DEFAULT_MAX_INFLIGHT_BYTES,
            queue: QueueLimits::default(),
            sender: SenderConfig::default(),
            send_timeout: Duration::from_secs(30),
            report_interval: Duration::from_secs(60),
            zstd_level: crate::wire::ZSTD_LEVEL,
            log_json: false,
            host_metrics_interval: None,
            host_metrics_root: PathBuf::from("/"),
            journal: None,
            generated_telemetry_tenant: None,
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Config, String> {
        let defaults = Config::default();
        let config = Config {
            listen_addr: socket_addr("COLLECTY_LISTEN_ADDR", defaults.listen_addr)?,
            data_dir: path("COLLECTY_DATA_DIR", defaults.data_dir),
            signy_url: string("COLLECTY_SIGNY_URL", defaults.signy_url),
            signy_access_client_id: optional_secret("COLLECTY_SIGNY_ACCESS_CLIENT_ID")?,
            signy_access_client_secret: optional_secret("COLLECTY_SIGNY_ACCESS_CLIENT_SECRET")?,
            max_request_bytes: bytes("COLLECTY_MAX_REQUEST_BYTES", defaults.max_request_bytes)?,
            max_inflight_bytes: bytes("COLLECTY_MAX_INFLIGHT_BYTES", defaults.max_inflight_bytes)?,
            queue: QueueLimits {
                max_bytes: bytes64("COLLECTY_QUEUE_MAX_BYTES", defaults.queue.max_bytes)?,
                max_segment_bytes: bytes64(
                    "COLLECTY_QUEUE_SEGMENT_BYTES",
                    defaults.queue.max_segment_bytes,
                )?,
                max_segment_age: duration(
                    "COLLECTY_SEGMENT_MAX_AGE",
                    defaults.queue.max_segment_age,
                )?,
            },
            sender: SenderConfig {
                retry_initial: duration("COLLECTY_RETRY_INITIAL", defaults.sender.retry_initial)?,
                retry_max: duration("COLLECTY_RETRY_MAX", defaults.sender.retry_max)?,
            },
            send_timeout: duration("COLLECTY_SEND_TIMEOUT", defaults.send_timeout)?,
            report_interval: duration("COLLECTY_REPORT_INTERVAL", defaults.report_interval)?,
            zstd_level: level("COLLECTY_ZSTD_LEVEL", defaults.zstd_level)?,
            log_json: matches!(
                string("COLLECTY_LOG_FORMAT", "text".to_string()).as_str(),
                "json"
            ),
            host_metrics_interval: optional_duration("COLLECTY_HOST_METRICS_INTERVAL")?,
            host_metrics_root: path("COLLECTY_HOST_METRICS_ROOT", defaults.host_metrics_root),
            journal: journal_config()?,
            generated_telemetry_tenant: tenant_id("COLLECTY_GENERATED_TELEMETRY_TENANT_ID")?,
        };
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        validate_signy_url(&self.signy_url)?;
        validate_access_credentials(
            &self.signy_access_client_id,
            &self.signy_access_client_secret,
        )?;
        if self.max_request_bytes > u32::MAX as usize {
            return Err(format!(
                "COLLECTY_MAX_REQUEST_BYTES is {} and cannot exceed {}",
                self.max_request_bytes,
                u32::MAX
            ));
        }
        if self.max_inflight_bytes < self.max_request_bytes {
            return Err(format!(
                "COLLECTY_MAX_INFLIGHT_BYTES ({}) is below COLLECTY_MAX_REQUEST_BYTES ({}), \
which would refuse every large export forever",
                self.max_inflight_bytes, self.max_request_bytes
            ));
        }
        if self.queue.max_bytes < self.queue.max_segment_bytes {
            return Err(format!(
                "COLLECTY_QUEUE_MAX_BYTES ({}) is below COLLECTY_QUEUE_SEGMENT_BYTES ({})",
                self.queue.max_bytes, self.queue.max_segment_bytes
            ));
        }
        if (self.max_request_bytes as u64) >= self.queue.max_bytes {
            return Err(format!(
                "COLLECTY_QUEUE_MAX_BYTES ({}) leaves no room for one \
COLLECTY_MAX_REQUEST_BYTES ({}) export",
                self.queue.max_bytes, self.max_request_bytes
            ));
        }
        if self.queue.max_segment_age.is_zero() {
            return Err("COLLECTY_SEGMENT_MAX_AGE must be positive".to_string());
        }
        if self
            .host_metrics_interval
            .is_some_and(|interval| interval.is_zero())
        {
            return Err("COLLECTY_HOST_METRICS_INTERVAL must be positive".to_string());
        }
        if !(1..=22).contains(&self.zstd_level) {
            return Err(format!(
                "COLLECTY_ZSTD_LEVEL is {} and must be between 1 and 22",
                self.zstd_level
            ));
        }
        if (self.host_metrics_interval.is_some() || self.journal.is_some())
            && self.generated_telemetry_tenant.is_none()
        {
            return Err(
                "COLLECTY_GENERATED_TELEMETRY_TENANT_ID is required when a collecty source is enabled"
                    .to_string(),
            );
        }
        Ok(())
    }

    pub fn queue_dir(&self) -> PathBuf {
        self.data_dir.join("queue")
    }
}

fn string(name: &str, fallback: String) -> String {
    std::env::var(name).unwrap_or(fallback)
}

fn optional_secret(name: &str) -> Result<Option<String>, String> {
    match std::env::var(name) {
        Err(_) => Ok(None),
        Ok(value) if value.is_empty() => Err(format!("{name} must not be empty")),
        Ok(value) => Ok(Some(value)),
    }
}

fn validate_signy_url(value: &str) -> Result<(), String> {
    let uri = value
        .parse::<http::Uri>()
        .map_err(|error| format!("invalid COLLECTY_SIGNY_URL: {error}"))?;
    let scheme = uri
        .scheme_str()
        .ok_or_else(|| "COLLECTY_SIGNY_URL must include http:// or https://".to_string())?;
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return Err(format!(
            "COLLECTY_SIGNY_URL uses unsupported scheme {scheme:?}; expected http or https"
        ));
    }
    let authority = uri
        .authority()
        .ok_or_else(|| "COLLECTY_SIGNY_URL must include a host".to_string())?;
    if authority.host().is_empty() {
        return Err("COLLECTY_SIGNY_URL must include a host".to_string());
    }
    if authority.as_str().contains('@') {
        return Err("COLLECTY_SIGNY_URL must not include user information".to_string());
    }
    if uri.query().is_some() {
        return Err("COLLECTY_SIGNY_URL must not include a query".to_string());
    }
    Ok(())
}

fn validate_access_credentials(
    access_client_id: &Option<String>,
    access_client_secret: &Option<String>,
) -> Result<(), String> {
    if access_client_id.is_some() != access_client_secret.is_some() {
        return Err(
            "COLLECTY_SIGNY_ACCESS_CLIENT_ID and COLLECTY_SIGNY_ACCESS_CLIENT_SECRET must be set together"
                .to_string(),
        );
    }
    for (name, value) in [
        ("COLLECTY_SIGNY_ACCESS_CLIENT_ID", access_client_id),
        ("COLLECTY_SIGNY_ACCESS_CLIENT_SECRET", access_client_secret),
    ] {
        if let Some(value) = value {
            HeaderValue::try_from(value)
                .map_err(|_| format!("{name} must be a valid HTTP header value"))?;
        }
    }
    Ok(())
}

/// The same grammar signy validates a tenant id against, checked here so a
/// typo fails startup rather than turning every generated export into a drop nobody
/// asked about.
fn tenant_id(name: &str) -> Result<Option<String>, String> {
    let Ok(value) = std::env::var(name) else {
        return Ok(None);
    };
    parse_tenant(&value)
        .map(Some)
        .map_err(|reason| format!("invalid {name} {value:?}: {reason}"))
}

fn parse_tenant(value: &str) -> Result<String, String> {
    if value.is_empty() {
        return Err("a tenant id must not be empty".to_string());
    }
    if value.len() > 64 {
        return Err(format!("{} bytes, over the 64 allowed", value.len()));
    }
    if let Some(invalid) = value
        .chars()
        .find(|c| !c.is_ascii_alphanumeric() && *c != '_' && *c != '-')
    {
        return Err(format!(
            "contains the unsupported character {invalid:?}; only [a-zA-Z0-9_-] is accepted"
        ));
    }
    Ok(value.to_string())
}

fn path(name: &str, fallback: PathBuf) -> PathBuf {
    std::env::var(name).map(PathBuf::from).unwrap_or(fallback)
}

fn socket_addr(name: &str, fallback: SocketAddr) -> Result<SocketAddr, String> {
    match std::env::var(name) {
        Err(_) => Ok(fallback),
        Ok(value) => value
            .parse()
            .map_err(|error| format!("invalid {name} {value:?}: {error}")),
    }
}

fn level(name: &str, fallback: i32) -> Result<i32, String> {
    match std::env::var(name) {
        Err(_) => Ok(fallback),
        Ok(value) => value
            .parse::<i32>()
            .map_err(|error| format!("invalid {name} {value:?}: {error}")),
    }
}

fn bytes(name: &str, fallback: usize) -> Result<usize, String> {
    Ok(bytes64(name, fallback as u64)? as usize)
}

fn bytes64(name: &str, fallback: u64) -> Result<u64, String> {
    match std::env::var(name) {
        Err(_) => Ok(fallback),
        Ok(value) => {
            let parsed = parse_bytes(&value)
                .ok_or_else(|| format!("invalid {name} {value:?}: expected 512, 64MiB or 1GiB"))?;
            if parsed == 0 {
                return Err(format!("{name} must be positive"));
            }
            Ok(parsed)
        }
    }
}

pub fn parse_bytes(value: &str) -> Option<u64> {
    let value = value.trim();
    let (digits, scale) = match value {
        _ if value.ends_with("KiB") => (&value[..value.len() - 3], 1024),
        _ if value.ends_with("MiB") => (&value[..value.len() - 3], 1024 * 1024),
        _ if value.ends_with("GiB") => (&value[..value.len() - 3], 1024 * 1024 * 1024),
        _ => (value, 1),
    };
    digits
        .trim()
        .parse::<u64>()
        .ok()
        .and_then(|number| number.checked_mul(scale))
}

fn duration(name: &str, fallback: Duration) -> Result<Duration, String> {
    match std::env::var(name) {
        Err(_) => Ok(fallback),
        Ok(value) => parse_duration(&value)
            .ok_or_else(|| format!("invalid {name} {value:?}: expected 500ms, 30s or 5m")),
    }
}

fn optional_duration(name: &str) -> Result<Option<Duration>, String> {
    match std::env::var(name) {
        Err(_) => Ok(None),
        Ok(value) => parse_duration(&value)
            .map(Some)
            .ok_or_else(|| format!("invalid {name} {value:?}: expected 500ms, 30s or 5m")),
    }
}

fn journal_config() -> Result<Option<JournalConfig>, String> {
    if !boolean("COLLECTY_JOURNAL", false)? {
        return Ok(None);
    }
    let directory = match std::env::var("COLLECTY_JOURNAL_DIRECTORY") {
        Ok(value) if !value.trim().is_empty() => Some(PathBuf::from(value)),
        Ok(_) => return Err("COLLECTY_JOURNAL_DIRECTORY must not be empty".to_string()),
        Err(_) => None,
    };
    let units = list("COLLECTY_JOURNAL_UNITS", 64, 128)?;
    let min_priority = priority("COLLECTY_JOURNAL_MIN_PRIORITY", 6)?;
    Ok(Some(JournalConfig {
        directory,
        journalctl_path: path("COLLECTY_JOURNALCTL_PATH", PathBuf::from("journalctl")),
        units,
        min_priority,
    }))
}

fn boolean(name: &str, fallback: bool) -> Result<bool, String> {
    match std::env::var(name) {
        Err(_) => Ok(fallback),
        Ok(value) => parse_boolean(&value)
            .ok_or_else(|| format!("invalid {name} {value:?}: expected true or false")),
    }
}

fn parse_boolean(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn list(name: &str, max_items: usize, max_item_bytes: usize) -> Result<Vec<String>, String> {
    let Ok(value) = std::env::var(name) else {
        return Ok(Vec::new());
    };
    let mut values = Vec::new();
    for item in value.split(',') {
        let item = item.trim();
        if item.is_empty() {
            return Err(format!(
                "invalid {name} {value:?}: entries must not be empty"
            ));
        }
        if item.len() > max_item_bytes {
            return Err(format!(
                "invalid {name}: an entry is over the {max_item_bytes} byte maximum"
            ));
        }
        values.push(item.to_string());
        if values.len() > max_items {
            return Err(format!("invalid {name}: more than {max_items} entries"));
        }
    }
    Ok(values)
}

fn priority(name: &str, fallback: u8) -> Result<u8, String> {
    let Ok(value) = std::env::var(name) else {
        return Ok(fallback);
    };
    let parsed = parse_priority(value.trim())
        .ok_or_else(|| format!("invalid {name} {value:?}: expected 0-7 or info"))?;
    Ok(parsed)
}

fn parse_priority(value: &str) -> Option<u8> {
    let normalized = value.trim().to_ascii_lowercase();
    let parsed = match normalized.as_str() {
        "emerg" => 0,
        "alert" => 1,
        "crit" => 2,
        "err" | "error" => 3,
        "warning" | "warn" => 4,
        "notice" => 5,
        "info" => 6,
        "debug" => 7,
        _ => normalized.parse::<u8>().ok()?,
    };
    (parsed <= 7).then_some(parsed)
}

pub fn parse_duration(value: &str) -> Option<Duration> {
    let value = value.trim();
    let (digits, unit) = match value {
        _ if value.ends_with("ms") => (&value[..value.len() - 2], 1),
        _ if value.ends_with('s') => (&value[..value.len() - 1], 1000),
        _ if value.ends_with('m') => (&value[..value.len() - 1], 60 * 1000),
        _ if value.ends_with('h') => (&value[..value.len() - 1], 60 * 60 * 1000),
        _ => return None,
    };
    digits
        .trim()
        .parse::<u64>()
        .ok()
        .and_then(|number| number.checked_mul(unit))
        .map(Duration::from_millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_sizes_accept_a_binary_suffix_and_reject_anything_else() {
        assert_eq!(parse_bytes("512"), Some(512));
        assert_eq!(parse_bytes("64KiB"), Some(64 * 1024));
        assert_eq!(parse_bytes("8 MiB"), Some(8 * 1024 * 1024));
        assert_eq!(parse_bytes("1GiB"), Some(1024 * 1024 * 1024));
        assert_eq!(parse_bytes("1GB"), None);
        assert_eq!(parse_bytes("many"), None);
    }

    /// The same grammar signy parses one with. Checked here so a typo fails
    /// startup: signy would answer 200 and drop every self-export, which is
    /// the one refusal shape that reports nothing back.
    #[test]
    fn a_tenant_id_is_validated_the_way_signy_validates_one() {
        assert_eq!(parse_tenant("fn0-proj_42").as_deref(), Ok("fn0-proj_42"));
        assert_eq!(
            parse_tenant(&"a".repeat(64)).as_deref(),
            Ok("a".repeat(64).as_str())
        );

        assert!(parse_tenant("").is_err());
        assert!(parse_tenant(&"a".repeat(65)).is_err());
        assert!(parse_tenant("a/b").is_err());
        assert!(parse_tenant("a b").is_err());
        assert!(parse_tenant("문자").is_err());
    }

    #[test]
    fn durations_need_a_unit() {
        assert_eq!(parse_duration("500ms"), Some(Duration::from_millis(500)));
        assert_eq!(parse_duration("30s"), Some(Duration::from_secs(30)));
        assert_eq!(parse_duration("5m"), Some(Duration::from_secs(300)));
        assert_eq!(parse_duration("2h"), Some(Duration::from_secs(7200)));
        assert_eq!(parse_duration("30"), None);
    }

    #[test]
    fn source_values_have_bounded_parsers() {
        assert_eq!(parse_boolean("yes"), Some(true));
        assert_eq!(parse_boolean("off"), Some(false));
        assert_eq!(parse_boolean("sometimes"), None);
        assert_eq!(parse_priority("warning"), Some(4));
        assert_eq!(parse_priority("7"), Some(7));
        assert_eq!(parse_priority("8"), None);
        assert_eq!(
            list("COLLECTY_TEST_UNSET_LIST", 4, 16).expect("unset"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn an_inflight_ceiling_below_the_request_ceiling_is_refused() {
        let config = Config {
            max_inflight_bytes: 1024,
            max_request_bytes: 4096,
            ..Config::default()
        };
        let error = config.validate().expect_err("a refusal");
        assert!(error.contains("COLLECTY_MAX_INFLIGHT_BYTES"), "{error}");
    }

    #[test]
    fn a_queue_that_cannot_hold_one_export_is_refused() {
        let config = Config {
            queue: QueueLimits {
                max_bytes: 4096,
                max_segment_bytes: 4096,
                ..QueueLimits::default()
            },
            max_request_bytes: 8192,
            max_inflight_bytes: 8192,
            ..Config::default()
        };
        let error = config.validate().expect_err("a refusal");
        assert!(error.contains("COLLECTY_QUEUE_MAX_BYTES"), "{error}");
    }

    #[test]
    fn the_defaults_are_consistent() {
        Config::default().validate().expect("consistent defaults");
    }

    #[test]
    fn signy_accepts_http_and_https_urls_only() {
        validate_signy_url("http://127.0.0.1:3100").expect("http");
        validate_signy_url("https://signy.example.test").expect("https");
        validate_signy_url("HTTPS://signy.example.test").expect("case-insensitive scheme");
        assert!(validate_signy_url("ftp://signy.example.test").is_err());
        assert!(validate_signy_url("signy.example.test").is_err());
        assert!(validate_signy_url("https:///missing-host").is_err());
        assert!(validate_signy_url("https://user:pass@signy.example.test").is_err());
        assert!(validate_signy_url("https://signy.example.test?token=secret").is_err());
    }

    #[test]
    fn access_credentials_are_optional_but_must_be_a_pair_of_headers() {
        validate_access_credentials(&None, &None).expect("unset");
        validate_access_credentials(
            &Some("client-id".to_string()),
            &Some("client-secret".to_string()),
        )
        .expect("valid headers");
        assert!(validate_access_credentials(&Some("client-id".to_string()), &None).is_err());
        assert!(validate_access_credentials(&None, &Some("client-secret".to_string())).is_err());
        assert!(
            validate_access_credentials(
                &Some("client\n-id".to_string()),
                &Some("client-secret".to_string())
            )
            .is_err()
        );
    }

    #[test]
    fn config_debug_redacts_access_credentials() {
        let config = Config {
            signy_access_client_id: Some("client-id".to_string()),
            signy_access_client_secret: Some("client-secret".to_string()),
            ..Config::default()
        };
        let debug = format!("{config:?}");
        assert!(!debug.contains("client-id"));
        assert!(!debug.contains("client-secret"));
        assert_eq!(debug.matches("[REDACTED]").count(), 2);
    }

    #[test]
    fn a_source_needs_a_tenant_and_a_positive_interval() {
        let missing_tenant = Config {
            host_metrics_interval: Some(Duration::from_secs(15)),
            ..Config::default()
        };
        assert!(
            missing_tenant
                .validate()
                .expect_err("missing tenant")
                .contains("COLLECTY_GENERATED_TELEMETRY_TENANT_ID")
        );

        let zero_interval = Config {
            host_metrics_interval: Some(Duration::ZERO),
            generated_telemetry_tenant: Some("host".to_string()),
            ..Config::default()
        };
        assert!(
            zero_interval
                .validate()
                .expect_err("zero interval")
                .contains("COLLECTY_HOST_METRICS_INTERVAL")
        );
    }
}
