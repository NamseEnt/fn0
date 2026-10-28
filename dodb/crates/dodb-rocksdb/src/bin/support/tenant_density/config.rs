use std::path::{Path, PathBuf};

use dodb_server::BlinkCheckpointPolicy;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Backend {
    Blink,
    MinimalNative,
}

impl Backend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Blink => "blink",
            Self::MinimalNative => "minimal-native",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Topology {
    FixedTotal,
    FixedPerTenant,
}

impl Topology {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FixedTotal => "fixed-total",
            Self::FixedPerTenant => "fixed-per-tenant",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Profile {
    ReadHeavy,
    Mixed,
    WriteHeavy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Width {
    Mixed,
    Fixed(usize),
}

impl Width {
    pub fn for_operation(self, operation_number: u64) -> usize {
        match self {
            Self::Fixed(width) => width,
            Self::Mixed => match operation_number % 100 {
                0..=79 => 1,
                80..=94 => 4,
                _ => 16,
            },
        }
    }
}

impl Profile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadHeavy => "read-heavy",
            Self::Mixed => "mixed",
            Self::WriteHeavy => "write-heavy",
        }
    }

    pub fn weights(self) -> [u8; 3] {
        match self {
            Self::ReadHeavy => [80, 15, 5],
            Self::Mixed => [40, 20, 40],
            Self::WriteHeavy => [10, 10, 80],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Options {
    pub backend: Backend,
    pub data_dir: PathBuf,
    pub report_path: PathBuf,
    pub seed: u64,
    pub registered_tenants: usize,
    pub active_tenants: usize,
    pub opened_idle_tenants: usize,
    pub client_count: usize,
    pub duration_ms: Option<u64>,
    pub operation_count: Option<u64>,
    pub initial_rows: usize,
    pub value_bytes: usize,
    pub topology: Topology,
    pub per_tenant_rate: Option<u64>,
    pub queue_bound: usize,
    pub max_in_flight: usize,
    pub request_timeout_ms: u64,
    pub drain_timeout_ms: u64,
    pub profile: Profile,
    pub width: Width,
    pub checkpoint_policy: BlinkCheckpointPolicy,
    pub blink_checkpoint: crate::blink_checkpoint::Options,
}

impl Options {
    pub fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut values = std::collections::BTreeMap::new();
        let mut arguments_iter = arguments.iter();
        while let Some(name) = arguments_iter.next() {
            if !name.starts_with("--") {
                return Err(format!("unexpected positional argument {name}"));
            }
            let value = arguments_iter
                .next()
                .ok_or_else(|| format!("missing value for {name}"))?;
            if values.insert(name.as_str(), value.as_str()).is_some() {
                return Err(format!("duplicate option {name}"));
            }
        }
        let known_options = [
            "--backend",
            "--data-dir",
            "--report",
            "--seed",
            "--registered-tenants",
            "--active-tenants",
            "--opened-idle-tenants",
            "--client-count",
            "--duration-ms",
            "--operation-count",
            "--initial-rows",
            "--value-bytes",
            "--topology",
            "--per-tenant-rate",
            "--queue-bound",
            "--max-in-flight",
            "--request-timeout-ms",
            "--drain-timeout-ms",
            "--profile",
            "--width",
            "--blink-wal-layout",
            "--blink-checkpoint-execution",
            "--blink-wal-segment-bytes",
            "--blink-checkpoint-wal-bytes",
        ];
        for name in values.keys() {
            if !known_options.contains(name) {
                return Err(format!("unknown option {name}"));
            }
        }
        let backend = match required(&values, "--backend")? {
            "blink" => Backend::Blink,
            "minimal-native" => Backend::MinimalNative,
            value => return Err(format!("unsupported backend {value}")),
        };
        let data_dir = PathBuf::from(required(&values, "--data-dir")?);
        let report_path = PathBuf::from(required(&values, "--report")?);
        let seed = parse_number(&values, "--seed")?;
        let registered_tenants = parse_number(&values, "--registered-tenants")?;
        let active_tenants = parse_number(&values, "--active-tenants")?;
        let opened_idle_tenants = parse_number(&values, "--opened-idle-tenants")?;
        let client_count = parse_number(&values, "--client-count")?;
        let duration_ms = parse_optional_number(&values, "--duration-ms")?;
        let operation_count = parse_optional_number(&values, "--operation-count")?;
        let initial_rows = parse_number(&values, "--initial-rows")?;
        let value_bytes = parse_number(&values, "--value-bytes")?;
        let topology = match required(&values, "--topology")? {
            "fixed-total" => Topology::FixedTotal,
            "fixed-per-tenant" => Topology::FixedPerTenant,
            value => return Err(format!("unsupported load topology {value}")),
        };
        let per_tenant_rate = parse_optional_number(&values, "--per-tenant-rate")?;
        let queue_bound = parse_number(&values, "--queue-bound")?;
        let max_in_flight = parse_number(&values, "--max-in-flight")?;
        let request_timeout_ms = parse_number(&values, "--request-timeout-ms")?;
        let drain_timeout_ms = parse_number(&values, "--drain-timeout-ms")?;
        let profile = match required(&values, "--profile")? {
            "read-heavy" => Profile::ReadHeavy,
            "mixed" => Profile::Mixed,
            "write-heavy" => Profile::WriteHeavy,
            value => return Err(format!("unsupported workload profile {value}")),
        };
        let width = match required(&values, "--width")? {
            "mixed" => Width::Mixed,
            "1" => Width::Fixed(1),
            "4" => Width::Fixed(4),
            "16" => Width::Fixed(16),
            value => {
                return Err(format!(
                    "unsupported write width {value}; expected mixed, 1, 4, or 16"
                ));
            }
        };
        let checkpoint_policy = match values.get("--blink-checkpoint-wal-bytes").copied() {
            None | Some("off") => BlinkCheckpointPolicy::Disabled,
            Some(value) => {
                let threshold_bytes = value.parse::<u64>().map_err(|_| {
                    "--blink-checkpoint-wal-bytes must be off or a positive byte count".to_owned()
                })?;
                if threshold_bytes == 0 {
                    return Err(
                        "--blink-checkpoint-wal-bytes must be off or a positive byte count"
                            .to_owned(),
                    );
                }
                BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes }
            }
        };
        let checkpoint_option_seen = values.contains_key("--blink-checkpoint-wal-bytes");
        let blink_checkpoint = crate::blink_checkpoint::Options::parse(
            values.get("--blink-wal-layout").copied(),
            values.get("--blink-checkpoint-execution").copied(),
            values.get("--blink-wal-segment-bytes").copied(),
            backend == Backend::Blink,
        )?;
        if checkpoint_option_seen && backend != Backend::Blink {
            return Err("--blink-checkpoint-wal-bytes requires --backend blink".to_owned());
        }
        let options = Self {
            backend,
            data_dir,
            report_path,
            seed,
            registered_tenants,
            active_tenants,
            opened_idle_tenants,
            client_count,
            duration_ms,
            operation_count,
            initial_rows,
            value_bytes,
            topology,
            per_tenant_rate,
            queue_bound,
            max_in_flight,
            request_timeout_ms,
            drain_timeout_ms,
            profile,
            width,
            checkpoint_policy,
            blink_checkpoint,
        };
        options.validate()?;
        Ok(options)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.registered_tenants == 0 || self.registered_tenants > 65_536 {
            return Err("registered tenants must be between 1 and 65536".to_owned());
        }
        if self.active_tenants == 0 {
            return Err("active tenants must be nonzero".to_owned());
        }
        if self
            .active_tenants
            .checked_add(self.opened_idle_tenants)
            .is_none_or(|opened| opened > self.registered_tenants)
        {
            return Err(
                "active plus opened-idle tenants cannot exceed registered tenants".to_owned(),
            );
        }
        if self.client_count == 0 || self.client_count > 65_536 {
            return Err("client count must be between 1 and 65536".to_owned());
        }
        match (self.duration_ms, self.operation_count) {
            (Some(duration), None) if duration > 0 && duration <= 86_400_000 => {}
            (None, Some(count)) if count > 0 => {}
            (Some(_), None) => return Err("duration must be between 1 ms and 24 hours".to_owned()),
            (None, Some(_)) => return Err("operation count must be nonzero".to_owned()),
            (Some(_), Some(_)) => {
                return Err("duration and operation count are mutually exclusive".to_owned());
            }
            (None, None) => return Err("one of duration or operation count is required".to_owned()),
        }
        if self.initial_rows < 256
            || self.initial_rows > 1_000_000
            || !self.initial_rows.is_multiple_of(256)
        {
            return Err(
                "initial rows must be a multiple of 256 between 256 and 1000000 per tenant"
                    .to_owned(),
            );
        }
        let Some(reserved_rows) = self.client_count.checked_mul(16) else {
            return Err(
                "client mutation key reservation overflows the supported key space".to_owned(),
            );
        };
        if self
            .initial_rows
            .checked_add(reserved_rows)
            .and_then(|first_preflight_row| first_preflight_row.checked_add(18))
            .is_none()
        {
            return Err("seed, client lanes, and preflight key ranges overflow".to_owned());
        }
        if !(1..=1_048_576).contains(&self.value_bytes) {
            return Err("value bytes must be between 1 and 1048576".to_owned());
        }
        match (self.topology, self.per_tenant_rate) {
            (Topology::FixedTotal, None) => {}
            (Topology::FixedTotal, Some(_)) => {
                return Err("per-tenant rate only applies to fixed-per-tenant topology".to_owned());
            }
            (Topology::FixedPerTenant, Some(rate)) if rate > 0 && rate <= 1_000_000 => {}
            (Topology::FixedPerTenant, Some(_)) => {
                return Err("per-tenant rate must be between 1 and 1000000".to_owned());
            }
            (Topology::FixedPerTenant, None) => {
                return Err("fixed-per-tenant topology requires per-tenant rate".to_owned());
            }
        }
        if self.queue_bound == 0 || self.queue_bound > 1_000_000 {
            return Err("queue bound must be between 1 and 1000000".to_owned());
        }
        if self.max_in_flight == 0 || self.max_in_flight > 65_536 {
            return Err("max in-flight must be between 1 and 65536".to_owned());
        }
        if self.request_timeout_ms == 0 || self.request_timeout_ms > 300_000 {
            return Err("request timeout must be between 1 and 300000 ms".to_owned());
        }
        if self.drain_timeout_ms == 0 || self.drain_timeout_ms > 3_600_000 {
            return Err("drain timeout must be between 1 and 3600000 ms".to_owned());
        }
        validate_paths(&self.data_dir, &self.report_path)?;
        Ok(())
    }
}

fn required<'values>(
    values: &'values std::collections::BTreeMap<&str, &str>,
    name: &str,
) -> Result<&'values str, String> {
    values
        .get(name)
        .copied()
        .ok_or_else(|| format!("missing required option {name}"))
}

fn parse_number<T: std::str::FromStr>(
    values: &std::collections::BTreeMap<&str, &str>,
    name: &str,
) -> Result<T, String> {
    required(values, name)?
        .parse()
        .map_err(|_| format!("invalid numeric value for {name}"))
}

fn parse_optional_number<T: std::str::FromStr>(
    values: &std::collections::BTreeMap<&str, &str>,
    name: &str,
) -> Result<Option<T>, String> {
    values
        .get(name)
        .map(|value| {
            value
                .parse()
                .map_err(|_| format!("invalid numeric value for {name}"))
        })
        .transpose()
}

fn validate_paths(data_dir: &Path, report_path: &Path) -> Result<(), String> {
    if !data_dir.is_absolute() || !report_path.is_absolute() {
        return Err("data and report paths must be absolute".to_owned());
    }
    if data_dir == report_path || report_path.starts_with(data_dir) {
        return Err("report path must be outside the database data directory".to_owned());
    }
    if data_dir.exists() {
        let metadata = std::fs::symlink_metadata(data_dir).map_err(|error| error.to_string())?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err("data path must be a real directory".to_owned());
        }
        if std::fs::read_dir(data_dir)
            .map_err(|error| error.to_string())?
            .next()
            .is_some()
        {
            return Err("data directory must be empty".to_owned());
        }
    }
    if report_path.exists() {
        return Err("report path already exists".to_owned());
    }
    if !report_path
        .parent()
        .is_some_and(|parent| parent.exists() && parent.is_dir())
    {
        return Err("report parent directory must already exist".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Backend, Options, Profile, Topology, Width};
    use tempfile::TempDir;

    fn arguments(extra: &[&str]) -> (TempDir, Vec<String>) {
        let directory = TempDir::new().unwrap();
        let data_dir = directory.path().join("database");
        let report_path = directory.path().join("report.json");
        let mut arguments = vec![
            "--backend",
            "blink",
            "--data-dir",
            data_dir.to_str().unwrap(),
            "--report",
            report_path.to_str().unwrap(),
            "--seed",
            "7",
            "--registered-tenants",
            "512",
            "--active-tenants",
            "512",
            "--opened-idle-tenants",
            "0",
            "--client-count",
            "8",
            "--operation-count",
            "100",
            "--initial-rows",
            "256",
            "--value-bytes",
            "512",
            "--topology",
            "fixed-total",
            "--queue-bound",
            "8",
            "--max-in-flight",
            "16",
            "--request-timeout-ms",
            "1000",
            "--drain-timeout-ms",
            "10000",
            "--profile",
            "read-heavy",
            "--width",
            "1",
        ];
        arguments.extend_from_slice(extra);
        (
            directory,
            arguments.into_iter().map(str::to_owned).collect(),
        )
    }

    #[test]
    fn parses_512_all_active_tenants_and_supported_value_sizes() {
        for value_size in ["512", "4096"] {
            let (directory, mut argument_values) = arguments(&[]);
            let value_position = argument_values
                .iter()
                .position(|argument| argument == "--value-bytes")
                .unwrap();
            argument_values[value_position + 1] = value_size.to_owned();
            let options = Options::parse(&argument_values).unwrap();
            assert_eq!(options.backend, Backend::Blink);
            assert_eq!(options.registered_tenants, 512);
            assert_eq!(options.active_tenants, 512);
            assert_eq!(options.value_bytes, value_size.parse::<usize>().unwrap());
            assert_eq!(options.topology, Topology::FixedTotal);
            assert_eq!(options.profile, Profile::ReadHeavy);
            drop(directory);
        }
    }

    #[test]
    fn fixed_total_allows_sixteen_lanes_with_minimum_seed_rows() {
        let (_directory, mut argument_values) = arguments(&[]);
        let client_position = argument_values
            .iter()
            .position(|argument| argument == "--client-count")
            .unwrap();
        argument_values[client_position + 1] = "16".to_owned();
        let options = Options::parse(&argument_values).unwrap();
        assert_eq!(options.client_count, 16);
        assert_eq!(options.initial_rows, 256);
    }

    #[test]
    fn rejects_unknown_duplicate_and_incompatible_configuration() {
        let (_directory, argument_values) = arguments(&["--unknown", "1"]);
        assert!(Options::parse(&argument_values).is_err());
        let (_directory, argument_values) = arguments(&["--seed", "8"]);
        assert!(Options::parse(&argument_values).is_err());
        let (_directory, argument_values) = arguments(&["--duration-ms", "1"]);
        assert!(Options::parse(&argument_values).is_err());
        let (_directory, argument_values) = arguments(&["--opened-idle-tenants", "1"]);
        assert!(Options::parse(&argument_values).is_err());
        let (_directory, argument_values) = arguments(&["--topology", "fixed-per-tenant"]);
        assert!(Options::parse(&argument_values).is_err());
        let (_directory, argument_values) =
            arguments(&["--topology", "fixed-per-tenant", "--per-tenant-rate", "10"]);
        assert!(Options::parse(&argument_values).is_err());
    }

    #[test]
    fn requires_a_rate_for_fixed_per_tenant_and_rejects_rate_for_fixed_total() {
        let (_directory, argument_values) = arguments(&["--per-tenant-rate", "10"]);
        assert!(Options::parse(&argument_values).is_err());
        let (_directory, mut argument_values) = arguments(&["--per-tenant-rate", "10"]);
        let topology_position = argument_values
            .iter()
            .position(|argument| argument == "--topology")
            .unwrap();
        argument_values[topology_position + 1] = "fixed-per-tenant".to_owned();
        let options = Options::parse(&argument_values);
        assert!(options.is_ok(), "{:?}", options.err());
        let (_directory, argument_values) =
            arguments(&["--topology", "fixed-total", "--per-tenant-rate", "10"]);
        assert!(Options::parse(&argument_values).is_err());
    }

    #[test]
    fn mixed_widths_follow_fixed_eighty_fifteen_five_schedule() {
        let width = Width::Mixed;
        let counts = (0..100).fold([0_u64; 3], |mut counts, operation_number| {
            match width.for_operation(operation_number) {
                1 => counts[0] += 1,
                4 => counts[1] += 1,
                16 => counts[2] += 1,
                _ => unreachable!(),
            }
            counts
        });
        assert_eq!(counts, [80, 15, 5]);
    }

    #[test]
    fn rejects_partial_query_partition() {
        let (_directory, mut argument_values) = arguments(&[]);
        let rows_position = argument_values
            .iter()
            .position(|argument| argument == "--initial-rows")
            .unwrap();
        argument_values[rows_position + 1] = "257".to_owned();
        assert!(Options::parse(&argument_values).is_err());
    }
}
