import datetime
import json
import pathlib
import subprocess
import sys
import time


INSTANCE_ID = "ocid1.instance.oc1.ap-osaka-1.anvwsljrrfkd6xqcuiozrid2bkb7uh6ba3p7zkwkip6kcagbwgd5cp7os2jq"
COMPARTMENT_ID = "ocid1.tenancy.oc1..aaaaaaaa7p7f5frocdgdqcctydyrrqv3kvr3jxmq6sxcr62fa5mcrqvtcy5a"
PROFILE = "OSAKA-VM"
REGION = "ap-osaka-1"
AVAILABILITY_DOMAIN = "IsRG:AP-OSAKA-1-AD-1"
SSH_KEY = str(pathlib.Path.home() / "Downloads" / "ssh-key-2026-09-23.key")
CONFIG_FILE = str(pathlib.Path.home() / ".oci" / "config")
SSH_HOST = "opc@217.142.246.204"
REMOTE_BASE = "/bench/btrfs/db/oci-a1-scaling-20260930"
RESULTS = pathlib.Path(__file__).resolve().parents[1]
EVENTS = RESULTS / "oci-timeline.jsonl"
EXPECTED_SHAPE = "VM.Standard.A1.Flex"
EXPECTED_MEMORY_GB = 12.0


def utc_now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def append_event(event):
    event["timestamp"] = utc_now()
    with EVENTS.open("a", encoding="utf-8") as event_file:
        event_file.write(json.dumps(event, sort_keys=True) + "\n")
        event_file.flush()


def oci(*arguments):
    command = [
        "oci", *arguments, "--profile", PROFILE, "--region", REGION,
        "--config-file", CONFIG_FILE, "--output", "json",
    ]
    result = subprocess.run(command, check=True, capture_output=True, text=True)
    return json.loads(result.stdout)


def get_instance():
    result = oci("compute", "instance", "get", "--instance-id", INSTANCE_ID)
    instance = result["data"]
    if instance["id"] != INSTANCE_ID or instance["shape"] != EXPECTED_SHAPE:
        raise RuntimeError(f"instance identity/shape changed: {instance.get('id')} {instance.get('shape')}")
    return instance


def attachment_snapshot():
    boot = oci(
        "compute", "boot-volume-attachment", "list",
        "--availability-domain", AVAILABILITY_DOMAIN,
        "--instance-id", INSTANCE_ID, "--compartment-id", COMPARTMENT_ID,
    )["data"]
    vnic = oci(
        "compute", "vnic-attachment", "list",
        "--instance-id", INSTANCE_ID, "--compartment-id", COMPARTMENT_ID,
    )["data"]
    vnics = []
    for item in vnic:
        vnic_info = oci("network", "vnic", "get", "--vnic-id", item["vnic-id"])["data"]
        vnics.append((item["id"], item["vnic-id"], vnic_info.get("private-ip"), vnic_info.get("public-ip")))
    return {
        "boot_volume_attachments": sorted((item["id"], item["boot-volume-id"]) for item in boot),
        "vnic_attachments": sorted(vnics),
    }


def require_state(instance, ocpus):
    shape_config = instance.get("shape-config") or {}
    if instance.get("lifecycle-state") != "RUNNING":
        raise RuntimeError(f"instance is not RUNNING: {instance.get('lifecycle-state')}")
    if float(shape_config.get("ocpus", -1)) != float(ocpus):
        raise RuntimeError(f"shape OCPU mismatch: {shape_config.get('ocpus')} expected {ocpus}")
    if float(shape_config.get("memory-in-gbs", -1)) != EXPECTED_MEMORY_GB:
        raise RuntimeError(f"shape memory changed: {shape_config.get('memory-in-gbs')}")


def resize(ocpus, expected_attachments):
    current = get_instance()
    current_ocpus = int(float(current["shape-config"]["ocpus"]))
    if current_ocpus == ocpus:
        require_state(current, ocpus)
        return current
    started = utc_now()
    append_event({"event": "resize_started", "from_ocpus": current_ocpus, "to_ocpus": ocpus, "instance_id": INSTANCE_ID, "started_at": started})
    oci(
        "compute", "instance", "update", "--instance-id", INSTANCE_ID,
        "--shape", EXPECTED_SHAPE,
        "--shape-config", json.dumps({"ocpus": ocpus, "memoryInGBs": EXPECTED_MEMORY_GB}),
        "--update-operation-constraint", "ALLOW_DOWNTIME", "--force",
        "--wait-for-state", "RUNNING", "--wait-interval-seconds", "3", "--max-wait-seconds", "1200",
    )
    returned = utc_now()
    instance = get_instance()
    require_state(instance, ocpus)
    attachments = attachment_snapshot()
    if attachments != expected_attachments:
        raise RuntimeError(f"volume/VNIC attachment identity changed after resize: {attachments}")
    append_event({
        "event": "resize_running_returned", "from_ocpus": current_ocpus, "to_ocpus": ocpus,
        "instance_id": instance["id"], "running_returned_at": returned,
        "shape_config": instance["shape-config"], "attachments": attachments,
    })
    return instance


def run_stage(stage, ocpus):
    readiness_started = utc_now()
    append_event({"event": "ssh_readiness_wait_started", "stage": stage, "started_at": readiness_started})
    ready = False
    for attempt in range(90):
        probe = subprocess.run(
            ["ssh", "-i", SSH_KEY, "-o", "IdentitiesOnly=yes", "-o", "ConnectTimeout=3", SSH_HOST, "true"],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )
        if probe.returncode == 0:
            ready = True
            append_event({"event": "ssh_ready", "stage": stage, "attempt": attempt + 1, "ready_at": utc_now()})
            break
        time.sleep(1)
    if not ready:
        raise RuntimeError(f"SSH did not become ready for stage {stage} after 90 attempts")
    command = [
        "ssh", "-i", SSH_KEY, "-o", "IdentitiesOnly=yes", SSH_HOST,
        f"python3 {REMOTE_BASE}/harness/run_stage.py {stage} {ocpus}",
    ]
    log_path = RESULTS / f"stage-{stage}.log"
    started = utc_now()
    append_event({"event": "benchmark_started", "stage": stage, "ocpus": ocpus, "started_at": started})
    with log_path.open("w", encoding="utf-8") as log_file:
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1)
        assert process.stdout is not None
        for line in process.stdout:
            log_file.write(line)
            log_file.flush()
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue
            if event.get("event") in {"preflight", "run_end", "stage_complete"}:
                print(json.dumps({key: event.get(key) for key in ("event", "stage", "name", "exit_code", "timestamp", "filesystem_uuid", "nproc") if key in event}, sort_keys=True), flush=True)
        exit_code = process.wait()
    ended = utc_now()
    append_event({"event": "benchmark_ended", "stage": stage, "ocpus": ocpus, "ended_at": ended, "exit_code": exit_code, "log": str(log_path)})
    if exit_code:
        raise RuntimeError(f"benchmark stage {stage} failed with exit {exit_code}")


def main():
    if len(sys.argv) != 1:
        raise SystemExit("usage: run_oci_scale.py")
    baseline = get_instance()
    require_state(baseline, 2)
    original_attachments = attachment_snapshot()
    append_event({
        "event": "baseline_api_confirmed", "instance_id": baseline["id"], "shape": baseline["shape"],
        "shape_config": baseline["shape-config"], "attachments": original_attachments,
    })
    failure = None
    try:
        resize(4, original_attachments)
        run_stage("4", 4)
        resize(6, original_attachments)
        run_stage("6", 6)
    except BaseException as error:
        failure = error
        append_event({"event": "experiment_error", "error_type": type(error).__name__, "error": str(error)})
    finally:
        restore_error = None
        for attempt in range(3):
            try:
                current = get_instance()
                if float(current["shape-config"]["ocpus"]) != 2.0 or current["lifecycle-state"] != "RUNNING":
                    resize(2, original_attachments)
                else:
                    require_state(current, 2)
                if attachment_snapshot() != original_attachments:
                    raise RuntimeError("volume/VNIC attachments differ from the original snapshot during restore")
                restore_error = None
                break
            except BaseException as error:
                restore_error = error
                append_event({"event": "restore_retry", "attempt": attempt + 1, "error_type": type(error).__name__, "error": str(error)})
                if attempt < 2:
                    time.sleep(5 * (attempt + 1))
        if restore_error is not None:
            append_event({"event": "restore_error", "error_type": type(restore_error).__name__, "error": str(restore_error)})
            raise RuntimeError("failed to restore the instance to 2 OCPU after three attempts") from restore_error
        post_error = None
        try:
            restored = get_instance()
            require_state(restored, 2)
            append_event({"event": "restore_2_ocpu_confirmed", "instance_id": INSTANCE_ID, "shape_config": restored["shape-config"]})
        except BaseException as restore_error:
            append_event({"event": "restore_error", "error_type": type(restore_error).__name__, "error": str(restore_error)})
            raise
        try:
            run_stage("2-post", 2)
        except BaseException as error:
            post_error = error
            append_event({"event": "post_check_error", "error_type": type(error).__name__, "error": str(error)})
        restored = get_instance()
        require_state(restored, 2)
        if attachment_snapshot() != original_attachments:
            raise RuntimeError("volume/VNIC attachments differ from the original snapshot after post-check")
        append_event({"event": "final_api_confirmation", "instance_id": INSTANCE_ID, "shape": restored["shape"], "shape_config": restored["shape-config"], "lifecycle_state": restored["lifecycle-state"]})
        if post_error is not None:
            raise post_error
    if failure is not None:
        raise failure


if __name__ == "__main__":
    main()
