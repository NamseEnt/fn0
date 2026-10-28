use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use serde_json::{Value, json};

pub fn filesystem_snapshot(root: &Path) -> Result<Value, String> {
    if !root.exists() {
        return Ok(
            json!({"status": "not_created", "logical_file_bytes": null, "allocated_file_bytes": null}),
        );
    }
    let mut directories = vec![root.to_path_buf()];
    let mut identities = BTreeSet::new();
    let mut regular_file_paths = 0_u64;
    let mut distinct_files = 0_u64;
    let mut directory_count = 0_u64;
    let mut logical_bytes = 0_u64;
    let mut allocated_file_bytes = 0_u64;
    let mut allocated_directory_bytes = 0_u64;
    while let Some(directory) = directories.pop() {
        let metadata = fs::symlink_metadata(&directory).map_err(|error| error.to_string())?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err("filesystem accounting requires real directories".to_owned());
        }
        directory_count += 1;
        allocated_directory_bytes = allocated_directory_bytes
            .checked_add(metadata.blocks().saturating_mul(512))
            .ok_or("directory allocation counter overflow")?;
        for entry in fs::read_dir(&directory).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let metadata = fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
            if metadata.file_type().is_symlink() {
                return Err("filesystem accounting refuses symlinks".to_owned());
            }
            if metadata.is_dir() {
                directories.push(entry.path());
            } else if metadata.is_file() {
                regular_file_paths += 1;
                if identities.insert((metadata.dev(), metadata.ino())) {
                    distinct_files += 1;
                    logical_bytes = logical_bytes
                        .checked_add(metadata.len())
                        .ok_or("logical file counter overflow")?;
                    allocated_file_bytes = allocated_file_bytes
                        .checked_add(metadata.blocks().saturating_mul(512))
                        .ok_or("file allocation counter overflow")?;
                }
            } else {
                return Err("filesystem accounting encountered a nonregular entry".to_owned());
            }
        }
    }
    Ok(json!({
        "status": "sampled",
        "logical_file_bytes": logical_bytes,
        "allocated_file_bytes": allocated_file_bytes,
        "allocated_directory_bytes": allocated_directory_bytes,
        "regular_file_paths": regular_file_paths,
        "distinct_files": distinct_files,
        "directory_count": directory_count,
        "duplicate_hardlink_paths": regular_file_paths - distinct_files,
        "allocation_source": "Unix st_blocks in 512-byte units, deduplicated by device/inode",
        "scope": "reachable files at this phase boundary; not continuous peak, ZFS dataset usage, or device writes",
        "unlinked_open_files": {"status": "unsupported", "bytes": null},
    }))
}

fn live_backend(boundary: &Value) -> Option<&Value> {
    boundary["processes"]
        .as_array()?
        .iter()
        .rev()
        .find(|process| {
            process["role"] == "backend-process"
                && matches!(
                    process["status"].as_str(),
                    Some("sampled" | "smaps_rollup_unavailable")
                )
        })
}

fn same_identity(process: &Value, identity: &Value) -> bool {
    process["pid"].as_u64().is_some()
        && process["pid"] == identity["pid"]
        && process["start_time_ticks"].as_u64().is_some()
        && process["start_time_ticks"] == identity["start_time_ticks"]
}

pub fn summarize_measurement_processes(diagnostic: &Value, before: &Value, after: &Value) -> Value {
    let Some(identity) = live_backend(before) else {
        return json!({"status": "unavailable", "reason": "no live backend identity at offer start", "backend": null, "driver": null});
    };
    let identity_stable =
        live_backend(after).is_some_and(|process| same_identity(process, identity));
    let mut periodic_samples = diagnostic["process_memory_samples"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|sample| sample["phase"] == "offered-load")
        .collect::<Vec<_>>();
    let phase_periodic_count = periodic_samples.len();
    let periodic_count = periodic_samples
        .iter()
        .filter(|sample| {
            sample["processes"].as_array().is_some_and(|processes| {
                processes
                    .iter()
                    .any(|process| same_identity(process, identity))
            })
        })
        .count();
    periodic_samples.extend([before, after]);
    let mut backend_samples = Vec::new();
    let mut driver_samples = Vec::new();
    let mut timestamps = BTreeSet::new();
    for sample in periodic_samples {
        let Some(processes) = sample["processes"].as_array() else {
            continue;
        };
        if let Some(process) = processes
            .iter()
            .find(|process| same_identity(process, identity))
        {
            backend_samples.push(process);
            if let Some(timestamp) = sample["timestamp_unix_ms"].as_u64() {
                timestamps.insert(timestamp);
            }
        }
        if let Some(process) = processes
            .iter()
            .find(|process| process["role"] == "harness-process")
        {
            driver_samples.push(process);
        }
    }
    let timestamps = timestamps.into_iter().collect::<Vec<_>>();
    let maximum_gap = timestamps.windows(2).map(|pair| pair[1] - pair[0]).max();
    json!({
        "status": if !identity_stable { "identity_changed" } else if periodic_count == 0 { "boundary_samples_only" } else { "sampled" },
        "backend_pid": identity["pid"],
        "backend_start_time_ticks": identity["start_time_ticks"],
        "same_backend_identity_at_offer_boundaries": identity_stable,
        "periodic_offer_sample_count": periodic_count,
        "phase_periodic_sample_count_before_identity_filter": phase_periodic_count,
        "boundary_snapshots_requested": 2,
        "maximum_observed_gap_ms": maximum_gap,
        "process_samples_truncated": diagnostic["process_memory_samples_truncated"],
        "backend": summarize_fields(&backend_samples),
        "driver": summarize_fields(&driver_samples),
        "scope": "offer phase and its boundaries, separated by PID/start-time; no setup/oracle samples or engine-heap claim",
        "peak_scope": "sampled observations only; instantaneous peaks between samples can be missed",
        "vm_hwm_scope": "process lifetime high-water includes earlier measurement-worker startup and open phases",
        "field_sources": {"rss_kb": "VmRSS or smaps_rollup Rss", "pss_kb": "smaps_rollup Pss", "anonymous_kb": "smaps_rollup Anonymous", "file_kb": "smaps_rollup Pss_File, not file RSS", "swap_kb": "smaps_rollup Swap or VmSwap", "vm_hwm_kb": "VmHWM"},
    })
}

fn summarize_fields(samples: &[&Value]) -> Value {
    let mut fields = serde_json::Map::new();
    for field_name in [
        "rss_kb",
        "pss_kb",
        "anonymous_kb",
        "file_kb",
        "swap_kb",
        "vm_hwm_kb",
    ] {
        let values = samples
            .iter()
            .filter_map(|sample| sample[field_name].as_u64())
            .collect::<Vec<_>>();
        fields.insert(
            field_name.to_owned(),
            json!({
                "observed_max": values.iter().copied().max(),
                "available_observations": values.len(),
                "unavailable_observations": samples.len() - values.len(),
            }),
        );
    }
    json!({"sample_count": samples.len(), "fields": fields})
}

fn process_start_ticks(pid: u64) -> Option<u64> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    stat.rsplit_once(')')?
        .1
        .split_whitespace()
        .nth(19)?
        .parse()
        .ok()
}

pub fn process_io_from_boundary(boundary: &Value) -> Value {
    let Some(identity) = live_backend(boundary) else {
        return json!({"status": "unavailable", "reason": "no live backend identity", "counters": null});
    };
    let pid = identity["pid"].as_u64().unwrap_or(0);
    let start_ticks = identity["start_time_ticks"].as_u64();
    if start_ticks.is_none() || process_start_ticks(pid) != start_ticks {
        return json!({"status": "unavailable", "reason": "process identity changed", "counters": null});
    }
    let Ok(contents) = fs::read_to_string(format!("/proc/{pid}/io")) else {
        return json!({"status": "unsupported", "reason": "process I/O counters unavailable", "counters": null});
    };
    let counters = contents
        .lines()
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            let value = value.trim().parse::<u64>().ok()?;
            Some((name.to_owned(), json!(value)))
        })
        .collect::<serde_json::Map<_, _>>();
    if process_start_ticks(pid) != start_ticks {
        return json!({"status": "unavailable", "reason": "process identity changed during I/O sampling", "counters": null});
    }
    json!({
        "status": "sampled", "pid": pid, "start_time_ticks": start_ticks, "counters": counters,
        "scope": "raw Linux process I/O counters; runtime/transport included, buffered writeback may lag, not WAL-only bytes or device amplification",
    })
}

pub fn host_device_snapshot() -> Value {
    let device = std::env::var("DODB_MEMORY_VDEV_DEVICE").unwrap_or_else(|_| "sda4".to_owned());
    if device.is_empty()
        || !device
            .bytes()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, b'_' | b'-'))
    {
        return json!({"status": "unsupported", "reason": "invalid block device name", "read_bytes": null, "write_bytes": null});
    }
    let path = format!("/sys/class/block/{device}/stat");
    let fields = fs::read_to_string(&path).ok().and_then(|contents| {
        contents
            .split_whitespace()
            .map(str::parse::<u64>)
            .collect::<Result<Vec<_>, _>>()
            .ok()
    });
    let read_bytes = fields
        .as_ref()
        .and_then(|fields| fields.get(2))
        .and_then(|sectors| sectors.checked_mul(512));
    let write_bytes = fields
        .as_ref()
        .and_then(|fields| fields.get(6))
        .and_then(|sectors| sectors.checked_mul(512));
    json!({
        "status": if read_bytes.is_some() && write_bytes.is_some() { "sampled" } else { "unsupported" },
        "device": device, "source": path, "read_bytes": read_bytes, "write_bytes": write_bytes,
        "scope": "whole host device counters including unrelated I/O; differences are not per-backend write amplification",
    })
}

#[cfg(test)]
mod tests {
    use super::{filesystem_snapshot, summarize_measurement_processes};
    use serde_json::json;

    #[test]
    fn hardlinked_file_allocation_is_counted_once_and_symlinks_are_rejected() {
        let directory = tempfile::TempDir::new().unwrap();
        let file = directory.path().join("data");
        std::fs::write(&file, [7; 4096]).unwrap();
        std::fs::hard_link(&file, directory.path().join("hardlink")).unwrap();
        let snapshot = filesystem_snapshot(directory.path()).unwrap();
        assert_eq!(snapshot["logical_file_bytes"], 4096);
        assert_eq!(snapshot["distinct_files"], 1);
        assert_eq!(snapshot["duplicate_hardlink_paths"], 1);
        std::os::unix::fs::symlink(&file, directory.path().join("symlink")).unwrap();
        assert!(filesystem_snapshot(directory.path()).is_err());
    }

    #[test]
    fn missing_data_directory_is_not_reported_as_zero_bytes() {
        let directory = tempfile::TempDir::new().unwrap();
        let snapshot = filesystem_snapshot(&directory.path().join("missing")).unwrap();
        assert_eq!(snapshot["status"], "not_created");
        assert!(snapshot["logical_file_bytes"].is_null());
    }

    #[test]
    fn measurement_summary_excludes_setup_and_reused_pids_and_keeps_unknown_pss() {
        let backend = json!({"role": "backend-process", "pid": 20, "start_time_ticks": 7, "status": "sampled", "rss_kb": 42});
        let before = json!({"timestamp_unix_ms": 100, "processes": [backend.clone()]});
        let after = json!({"timestamp_unix_ms": 160, "processes": [backend]});
        let diagnostic = json!({"process_samples_truncated": false, "process_memory_samples": [
            {"phase": "setup", "timestamp_unix_ms": 90, "processes": [{"pid": 10, "start_time_ticks": 5, "rss_kb": 9000}]},
            {"phase": "offered-load", "timestamp_unix_ms": 120, "processes": [{"pid": 20, "start_time_ticks": 99, "rss_kb": 8000}]}
        ]});
        let summary = summarize_measurement_processes(&diagnostic, &before, &after);
        assert_eq!(summary["backend"]["fields"]["rss_kb"]["observed_max"], 42);
        assert!(summary["backend"]["fields"]["pss_kb"]["observed_max"].is_null());
        assert_eq!(summary["maximum_observed_gap_ms"], 60);
        assert_eq!(summary["periodic_offer_sample_count"], 0);
        assert_eq!(summary["status"], "boundary_samples_only");
    }

    #[test]
    fn short_offer_reports_boundary_only_sampling() {
        let boundary = json!({"timestamp_unix_ms": 100, "processes": [{"role": "backend-process", "pid": 20, "start_time_ticks": 7, "status": "sampled", "rss_kb": 42}]});
        let summary = summarize_measurement_processes(&json!({}), &boundary, &boundary);
        assert_eq!(summary["status"], "boundary_samples_only");
        assert_eq!(summary["periodic_offer_sample_count"], 0);
        assert!(summary["maximum_observed_gap_ms"].is_null());
    }
}
