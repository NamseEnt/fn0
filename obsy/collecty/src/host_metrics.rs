use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "linux")]
use std::ffi::CString;
#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStrExt;

use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::common::v1::{AnyValue, InstrumentationScope, KeyValue, any_value};
use opentelemetry_proto::tonic::metrics::v1::{
    AggregationTemporality, Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum,
    metric, number_data_point,
};
use opentelemetry_proto::tonic::resource::v1::Resource;
use prost::Message;

use crate::TENANT_ATTRIBUTE;

const MAX_DEVICES: usize = 256;
const MAX_FILESYSTEMS: usize = 128;
const MAX_INTERFACES: usize = 256;
#[derive(Clone, Debug)]
pub struct HostMetrics {
    root: PathBuf,
    tenant_id: String,
    host_name: String,
    boot_time_unix_nanos: u64,
    ticks_per_second: f64,
}

#[derive(Clone, Debug)]
struct Point {
    name: &'static str,
    unit: &'static str,
    kind: Kind,
    value: Value,
    attributes: Vec<(&'static str, String)>,
}

#[derive(Clone, Copy, Debug)]
enum Kind {
    Gauge,
    Counter,
}

#[derive(Clone, Copy, Debug)]
enum Value {
    Integer(u64),
    Double(f64),
}

impl HostMetrics {
    pub fn new(root: impl Into<PathBuf>, tenant_id: impl Into<String>) -> io::Result<Self> {
        let root = root.into();
        let tenant_id = tenant_id.into();
        let host_name = read_trimmed(&root.join("etc/hostname"))?;
        let stat = read_required(&root, "proc/stat")?;
        for relative in [
            "proc/meminfo",
            "proc/loadavg",
            "proc/net/dev",
            "proc/diskstats",
        ] {
            let _ = read_required(&root, relative)?;
        }
        let boot_seconds = parse_boot_time(&stat)?;
        let ticks_per_second = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
        let ticks_per_second = if ticks_per_second > 0 {
            ticks_per_second as f64
        } else {
            100.0
        };
        Ok(Self {
            root,
            tenant_id,
            host_name,
            boot_time_unix_nanos: boot_seconds.saturating_mul(1_000_000_000),
            ticks_per_second,
        })
    }

    pub fn collect(&self) -> io::Result<Vec<u8>> {
        let now = unix_nanos();
        let mut points = Vec::new();
        points.extend(cpu_points(
            &read_required(&self.root, "proc/stat")?,
            self.ticks_per_second,
        ));
        points.extend(memory_points(&read_required(&self.root, "proc/meminfo")?));
        points.extend(load_points(&read_required(&self.root, "proc/loadavg")?));
        points.extend(network_points(&read_required(&self.root, "proc/net/dev")?));
        points.extend(disk_points(&read_required(&self.root, "proc/diskstats")?));
        points.extend(filesystem_points(&self.root));
        Ok(encode(
            points,
            self.boot_time_unix_nanos,
            now,
            &self.host_name,
            &self.tenant_id,
        ))
    }

    pub fn host_name(&self) -> &str {
        &self.host_name
    }
}

fn encode(
    points: Vec<Point>,
    start_time: u64,
    now: u64,
    host_name: &str,
    tenant_id: &str,
) -> Vec<u8> {
    let metrics = points
        .into_iter()
        .map(|point| {
            let data_point = NumberDataPoint {
                start_time_unix_nano: if matches!(point.kind, Kind::Counter) {
                    start_time
                } else {
                    0
                },
                time_unix_nano: now,
                attributes: point
                    .attributes
                    .into_iter()
                    .map(|(key, value)| attribute(key, &value))
                    .chain([attribute("host.name", host_name)])
                    .collect(),
                value: Some(match point.value {
                    Value::Integer(value) => number_data_point::Value::AsInt(clamp_i64(value)),
                    Value::Double(value) => number_data_point::Value::AsDouble(value),
                }),
                ..Default::default()
            };
            Metric {
                name: point.name.to_string(),
                unit: point.unit.to_string(),
                data: Some(match point.kind {
                    Kind::Gauge => metric::Data::Gauge(Gauge {
                        data_points: vec![data_point],
                    }),
                    Kind::Counter => metric::Data::Sum(Sum {
                        data_points: vec![data_point],
                        aggregation_temporality: AggregationTemporality::Cumulative as i32,
                        is_monotonic: true,
                    }),
                }),
                ..Default::default()
            }
        })
        .collect();

    ExportMetricsServiceRequest {
        resource_metrics: vec![ResourceMetrics {
            resource: Some(Resource {
                attributes: vec![
                    attribute(TENANT_ATTRIBUTE, tenant_id),
                    attribute("service.name", "collecty"),
                    attribute("host.name", host_name),
                ],
                ..Default::default()
            }),
            scope_metrics: vec![ScopeMetrics {
                scope: Some(InstrumentationScope {
                    name: "collecty.host_metrics".to_string(),
                    ..Default::default()
                }),
                metrics,
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
    .encode_to_vec()
}

fn cpu_points(contents: &str, ticks_per_second: f64) -> Vec<Point> {
    let Some(line) = contents.lines().find(|line| line.starts_with("cpu ")) else {
        return Vec::new();
    };
    let modes = [
        "user", "nice", "system", "idle", "iowait", "irq", "softirq", "steal",
    ];
    line.split_whitespace()
        .skip(1)
        .zip(modes)
        .filter_map(|(raw, mode)| {
            raw.parse::<u64>().ok().map(|ticks| Point {
                name: "system.cpu.time",
                unit: "s",
                kind: Kind::Counter,
                value: Value::Double(ticks as f64 / ticks_per_second),
                attributes: vec![("cpu.mode", mode.to_string())],
            })
        })
        .collect()
}

fn memory_points(contents: &str) -> Vec<Point> {
    let mut total = 0;
    let mut available = 0;
    let mut swap_total = 0;
    let mut swap_free = 0;
    for line in contents.lines() {
        let mut fields = line.split_whitespace();
        let Some(name) = fields.next() else { continue };
        let Some(value) = fields.next().and_then(|value| value.parse::<u64>().ok()) else {
            continue;
        };
        let bytes = if fields.next() == Some("kB") {
            value.saturating_mul(1024)
        } else {
            value
        };
        match name {
            "MemTotal:" => total = bytes,
            "MemAvailable:" => available = bytes,
            "SwapTotal:" => swap_total = bytes,
            "SwapFree:" => swap_free = bytes,
            _ => {}
        }
    }
    vec![
        Point {
            name: "system.memory.usage",
            unit: "By",
            kind: Kind::Gauge,
            value: Value::Integer(total.saturating_sub(available)),
            attributes: vec![("state", "used".to_string())],
        },
        Point {
            name: "system.memory.usage",
            unit: "By",
            kind: Kind::Gauge,
            value: Value::Integer(available),
            attributes: vec![("state", "free".to_string())],
        },
        Point {
            name: "system.swap.usage",
            unit: "By",
            kind: Kind::Gauge,
            value: Value::Integer(swap_total.saturating_sub(swap_free)),
            attributes: vec![("state", "used".to_string())],
        },
        Point {
            name: "system.swap.usage",
            unit: "By",
            kind: Kind::Gauge,
            value: Value::Integer(swap_free),
            attributes: vec![("state", "free".to_string())],
        },
    ]
}

fn load_points(contents: &str) -> Vec<Point> {
    let Some(value) = contents
        .split_whitespace()
        .next()
        .and_then(|value| value.parse::<f64>().ok())
    else {
        return Vec::new();
    };
    vec![Point {
        name: "system.load_average.1m",
        unit: "{count}",
        kind: Kind::Gauge,
        value: Value::Double(value),
        attributes: Vec::new(),
    }]
}

fn network_points(contents: &str) -> Vec<Point> {
    contents
        .lines()
        .skip_while(|line| !line.contains(':'))
        .take(MAX_INTERFACES)
        .filter_map(|line| {
            let (interface, values) = line.split_once(':')?;
            let fields: Vec<_> = values.split_whitespace().collect();
            let received = fields.first()?.parse::<u64>().ok()?;
            let transmitted = fields.get(8)?.parse::<u64>().ok()?;
            Some([
                Point {
                    name: "system.network.io",
                    unit: "By",
                    kind: Kind::Counter,
                    value: Value::Integer(received),
                    attributes: vec![
                        ("interface", interface.trim().to_string()),
                        ("direction", "receive".to_string()),
                    ],
                },
                Point {
                    name: "system.network.io",
                    unit: "By",
                    kind: Kind::Counter,
                    value: Value::Integer(transmitted),
                    attributes: vec![
                        ("interface", interface.trim().to_string()),
                        ("direction", "transmit".to_string()),
                    ],
                },
            ])
        })
        .flatten()
        .collect()
}

fn disk_points(contents: &str) -> Vec<Point> {
    contents
        .lines()
        .take(MAX_DEVICES)
        .filter_map(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            if fields.len() < 10 {
                return None;
            }
            let device = fields.get(2)?.to_string();
            if device.starts_with("loop") || device.starts_with("ram") {
                return None;
            }
            let read_sectors = fields.get(5)?.parse::<u64>().ok()?;
            let write_sectors = fields.get(9)?.parse::<u64>().ok()?;
            Some([
                Point {
                    name: "system.disk.io",
                    unit: "By",
                    kind: Kind::Counter,
                    value: Value::Integer(read_sectors.saturating_mul(512)),
                    attributes: vec![
                        ("device", device.clone()),
                        ("direction", "read".to_string()),
                    ],
                },
                Point {
                    name: "system.disk.io",
                    unit: "By",
                    kind: Kind::Counter,
                    value: Value::Integer(write_sectors.saturating_mul(512)),
                    attributes: vec![("device", device), ("direction", "write".to_string())],
                },
            ])
        })
        .flatten()
        .collect()
}

fn filesystem_points(root: &Path) -> Vec<Point> {
    let Ok(contents) = fs::read_to_string(root.join("proc/mounts")) else {
        return Vec::new();
    };
    contents
        .lines()
        .take(MAX_FILESYSTEMS)
        .filter_map(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            let mountpoint = decode_mount_field(fields.get(1)?)?;
            let filesystem = fields.get(2)?;
            if matches!(
                *filesystem,
                "proc" | "sysfs" | "devtmpfs" | "cgroup" | "cgroup2" | "tmpfs"
            ) {
                return None;
            }
            let path = root.join(mountpoint.trim_start_matches('/'));
            let statistics = statvfs(&path).ok()?;
            let used = statistics.total.saturating_sub(statistics.available);
            Some([
                Point {
                    name: "system.filesystem.usage",
                    unit: "By",
                    kind: Kind::Gauge,
                    value: Value::Integer(used),
                    attributes: vec![
                        ("mountpoint", mountpoint.clone()),
                        ("state", "used".to_string()),
                    ],
                },
                Point {
                    name: "system.filesystem.usage",
                    unit: "By",
                    kind: Kind::Gauge,
                    value: Value::Integer(statistics.available),
                    attributes: vec![("mountpoint", mountpoint), ("state", "free".to_string())],
                },
            ])
        })
        .flatten()
        .collect()
}

struct VfsStats {
    total: u64,
    available: u64,
}

#[cfg(target_os = "linux")]
fn statvfs(path: &Path) -> io::Result<VfsStats> {
    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "a mount path contains NUL"))?;
    let mut statistics = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    let result = unsafe { libc::statvfs(path.as_ptr(), statistics.as_mut_ptr()) };
    if result != 0 {
        return Err(io::Error::last_os_error());
    }
    let statistics = unsafe { statistics.assume_init() };
    Ok(VfsStats {
        total: statistics.f_blocks.saturating_mul(statistics.f_frsize),
        available: statistics.f_bavail.saturating_mul(statistics.f_frsize),
    })
}

#[cfg(not(target_os = "linux"))]
fn statvfs(_path: &Path) -> io::Result<VfsStats> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "filesystem metrics are supported on Linux only",
    ))
}

fn parse_boot_time(contents: &str) -> io::Result<u64> {
    contents
        .lines()
        .find_map(|line| line.strip_prefix("btime ")?.trim().parse::<u64>().ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "proc/stat has no btime"))
}

fn read_required(root: &Path, relative: &str) -> io::Result<String> {
    fs::read_to_string(root.join(relative))
}

fn read_trimmed(path: &Path) -> io::Result<String> {
    let value = fs::read_to_string(path)?.trim().to_string();
    if value.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "hostname is empty",
        ));
    }
    Ok(value)
}

fn decode_mount_field(value: &str) -> Option<String> {
    let mut decoded = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut position = 0;
    while position < bytes.len() {
        if bytes[position] == b'\\' && position + 3 < bytes.len() {
            let digits = &bytes[position + 1..position + 4];
            if digits.iter().all(|digit| (b'0'..=b'7').contains(digit)) {
                let byte = (digits[0] - b'0') * 64 + (digits[1] - b'0') * 8 + digits[2] - b'0';
                decoded.push(byte as char);
                position += 4;
                continue;
            }
        }
        decoded.push(bytes[position] as char);
        position += 1;
    }
    Some(decoded)
}

fn attribute(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(value.to_string())),
        }),
        ..Default::default()
    }
}

fn clamp_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn unix_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::Scratch;
    use opentelemetry_proto::tonic::metrics::v1::metric;
    use prost::Message;

    #[test]
    fn fixture_proc_files_become_bounded_otlp_metrics() {
        let scratch = Scratch::new("host-metrics");
        std::fs::create_dir_all(scratch.path().join("proc/net")).expect("proc dirs");
        std::fs::create_dir_all(scratch.path().join("etc")).expect("etc dir");
        std::fs::write(scratch.path().join("etc/hostname"), "fixture-host\n").expect("hostname");
        std::fs::write(
            scratch.path().join("proc/stat"),
            "cpu  100 20 30 400 5 6 7 8 0 0\nbtime 1700000000\n",
        )
        .expect("stat");
        std::fs::write(
            scratch.path().join("proc/meminfo"),
            "MemTotal:       1000 kB\nMemAvailable:    400 kB\nSwapTotal:       200 kB\nSwapFree:         50 kB\n",
        )
        .expect("meminfo");
        std::fs::write(
            scratch.path().join("proc/loadavg"),
            "1.25 0.50 0.25 1/100 42\n",
        )
        .expect("loadavg");
        std::fs::write(
            scratch.path().join("proc/net/dev"),
            "Inter-| Receive | Transmit\n eth0: 10 0 0 0 0 0 0 0 20 0 0 0 0 0 0 0\n",
        )
        .expect("net");
        std::fs::write(
            scratch.path().join("proc/diskstats"),
            "8 0 sda 1 0 3 0 2 0 4 0 0 0 0\n",
        )
        .expect("disk");

        let source = HostMetrics::new(scratch.path(), "tenant").expect("source");
        let export =
            ExportMetricsServiceRequest::decode(source.collect().expect("export").as_slice())
                .expect("valid export");
        let resource = export.resource_metrics[0]
            .resource
            .as_ref()
            .expect("resource");
        assert!(
            resource
                .attributes
                .iter()
                .any(|item| item.key == TENANT_ATTRIBUTE)
        );
        assert!(
            resource
                .attributes
                .iter()
                .any(|item| item.key == "host.name")
        );
        let metrics = &export.resource_metrics[0].scope_metrics[0].metrics;
        assert!(metrics.iter().any(|item| item.name == "system.cpu.time"));
        assert!(
            metrics
                .iter()
                .any(|item| item.name == "system.memory.usage")
        );
        assert!(metrics.iter().any(|item| item.name == "system.network.io"));
        assert!(metrics.iter().any(|item| item.name == "system.disk.io"));
        let cpu = metrics
            .iter()
            .find(|item| item.name == "system.cpu.time")
            .expect("cpu");
        assert!(matches!(cpu.data, Some(metric::Data::Sum(_))));
        let Some(metric::Data::Sum(cpu_sum)) = &cpu.data else {
            panic!("cpu is not a sum");
        };
        assert!(cpu_sum.data_points.iter().all(|point| {
            point.attributes.iter().any(|attribute| {
                attribute.key == "host.name"
                    && attribute
                        .value
                        .as_ref()
                        .and_then(|value| value.value.as_ref())
                        .is_some_and(|value| {
                            matches!(value, any_value::Value::StringValue(name) if name == "fixture-host")
                        })
            })
        }));
    }

    #[test]
    fn mount_fields_decode_octal_escapes() {
        assert_eq!(
            decode_mount_field("/var/lib/my\\040data"),
            Some("/var/lib/my data".to_string())
        );
    }
}
