import argparse
import json
import signal
import time
from pathlib import Path

from bcc import BPF


parser = argparse.ArgumentParser()
parser.add_argument("--pid", type=int, required=True)
parser.add_argument("--output", type=Path, required=True)
options = parser.parse_args()
options.output.mkdir(parents=True, exist_ok=True)
syscalls = ["fsync", "fdatasync", "pwrite64", "pread64", "futex"]
program = r"""
#include <uapi/linux/ptrace.h>
struct wait_start { u64 timestamp; s32 stack_id; u32 state; };
struct wait_key { u64 second; u32 thread_id; s32 stack_id; u32 runnable; };
struct aggregate { u64 count; u64 total_ns; u64 max_ns; u64 errors; u64 bytes; };
struct call_start { u64 timestamp; };
struct call_key { u64 second; u32 thread_id; u32 kind; };
struct name_value { char text[16]; };
BPF_HASH(wait_starts, u32, struct wait_start, 4096);
BPF_HASH(waits, struct wait_key, struct aggregate, 65536);
BPF_HASH(calls, struct call_key, struct aggregate, 65536);
BPF_HASH(thread_names, u32, struct name_value, 4096);
BPF_STACK_TRACE(stacks, 8192);
BPF_ARRAY(diagnostics, u64, 2);

TRACEPOINT_PROBE(sched, sched_switch) {
    u64 timestamp = bpf_ktime_get_ns();
    u64 identity = bpf_get_current_pid_tgid();
    u32 thread_id = identity;
    if ((identity >> 32) == TARGET_PID) {
        struct wait_start entry = {};
        entry.timestamp = timestamp;
        entry.stack_id = stacks.get_stackid(args, 0);
        entry.state = args->prev_state;
        wait_starts.update(&thread_id, &entry);
        struct name_value name = {};
        bpf_get_current_comm(&name.text, sizeof(name.text));
        thread_names.update(&thread_id, &name);
    }
    u32 next_thread = args->next_pid;
    struct wait_start *entry = wait_starts.lookup(&next_thread);
    if (!entry) return 0;
    struct wait_key key = {};
    key.second = entry->timestamp / 1000000000;
    key.thread_id = next_thread;
    key.stack_id = entry->stack_id;
    key.runnable = entry->state == 0;
    struct aggregate empty = {};
    struct aggregate *value = waits.lookup_or_try_init(&key, &empty);
    if (value) {
        u64 duration = timestamp - entry->timestamp;
        __sync_fetch_and_add(&value->count, 1);
        __sync_fetch_and_add(&value->total_ns, duration);
        if (duration > value->max_ns) value->max_ns = duration;
    } else {
        u32 slot = 0;
        u64 *counter = diagnostics.lookup(&slot);
        if (counter) __sync_fetch_and_add(counter, 1);
    }
    wait_starts.delete(&next_thread);
    return 0;
}
""".replace("TARGET_PID", str(options.pid))
for kind, syscall in enumerate(syscalls):
    program += f"""
BPF_HASH(start_{syscall}, u32, struct call_start, 4096);
TRACEPOINT_PROBE(syscalls, sys_enter_{syscall}) {{
    u64 identity = bpf_get_current_pid_tgid();
    if ((identity >> 32) != {options.pid}) return 0;
    u32 thread_id = identity;
    struct call_start entry = {{}};
    entry.timestamp = bpf_ktime_get_ns();
    start_{syscall}.update(&thread_id, &entry);
    return 0;
}}
TRACEPOINT_PROBE(syscalls, sys_exit_{syscall}) {{
    u64 identity = bpf_get_current_pid_tgid();
    if ((identity >> 32) != {options.pid}) return 0;
    u32 thread_id = identity;
    struct call_start *entry = start_{syscall}.lookup(&thread_id);
    if (!entry) return 0;
    struct call_key key = {{}};
    key.second = entry->timestamp / 1000000000;
    key.thread_id = thread_id;
    key.kind = {kind};
    struct aggregate empty = {{}};
    struct aggregate *value = calls.lookup_or_try_init(&key, &empty);
    if (value) {{
        u64 duration = bpf_ktime_get_ns() - entry->timestamp;
        __sync_fetch_and_add(&value->count, 1);
        __sync_fetch_and_add(&value->total_ns, duration);
        if (duration > value->max_ns) value->max_ns = duration;
        if (args->ret < 0) __sync_fetch_and_add(&value->errors, 1);
        if (args->ret > 0 && ({kind} == 2 || {kind} == 3))
            __sync_fetch_and_add(&value->bytes, args->ret);
    }} else {{
        u32 slot = 1;
        u64 *counter = diagnostics.lookup(&slot);
        if (counter) __sync_fetch_and_add(counter, 1);
    }}
    start_{syscall}.delete(&thread_id);
    return 0;
}}
"""
(options.output / "program.c").write_text(program)
collector = BPF(text=program)
offset_ns = time.time_ns() - time.monotonic_ns()
(options.output / "ready.json").write_text(json.dumps({"pid": options.pid, "realtime_minus_monotonic_ns": offset_ns}))
running = True


def stop(signum, frame):
    global running
    running = False


signal.signal(signal.SIGTERM, stop)
signal.signal(signal.SIGINT, stop)
while running and not (options.output / "stop").exists():
    time.sleep(0.1)


def aggregate(value):
    return {field: int(getattr(value, field)) for field in ["count", "total_ns", "max_ns", "errors", "bytes"]}


names = {str(key.value): bytes(value.text).split(b"\0", 1)[0].decode(errors="replace")
         for key, value in collector["thread_names"].items()}
waits = []
stack_ids = set()
for key, value in collector["waits"].items():
    waits.append({"second": int(key.second), "thread_id": int(key.thread_id),
                  "stack_id": int(key.stack_id), "runnable": bool(key.runnable), **aggregate(value)})
    stack_ids.add(int(key.stack_id))
stacks = {}
for stack_id in stack_ids:
    stacks[str(stack_id)] = ([collector.ksym(address).decode(errors="replace")
                              for address in collector["stacks"].walk(stack_id)]
                             if stack_id >= 0 else [f"stack_capture_error_{stack_id}"])
calls = [{"second": int(key.second), "thread_id": int(key.thread_id),
          "syscall": syscalls[key.kind], **aggregate(value)}
         for key, value in collector["calls"].items()]
result = {"pid": options.pid, "realtime_minus_monotonic_ns": offset_ns,
          "final_realtime_minus_monotonic_ns": time.time_ns() - time.monotonic_ns(),
          "thread_names": names, "kernel_stacks": stacks, "waits": waits, "syscalls": calls,
          "map_insert_failures": [int(value.value) for value in collector["diagnostics"].values()]}
(options.output / "waits.json").write_text(json.dumps(result, indent=2) + "\n")
collector.cleanup()
