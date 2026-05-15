# fn0-ski

Minimal WinterCG-compatible JS runtime embedded in fn0. Built on `deno_core` (V8).
Bundle input is assumed to be fully resolved JS (no `require` / `import` /
module loader at runtime — use rolldown to bundle).

## Concurrency model

ski's runtime model is **N isolates per thread, M threads**:

- **Threads (M).** fn0-worker spawns one OS thread per CPU core. Each thread
  owns a `current_thread` tokio runtime with its own `LocalSet`.
- **Isolates (N per thread).** A single thread may host multiple `SkiInstance`s
  simultaneously. Different projects hash-bucket onto a thread, and bundle
  hot-swap briefly keeps the old `SkiInstance` alive next to the new one on
  the same thread.
- **Isolate ↔ thread affinity is permanent.** A `SkiInstance` is `!Send`:
  once created on a thread it lives and dies there. The type system enforces
  this — there is no API that moves an isolate across threads. An isolate
  never yields to a different thread, only to other isolates on its own
  thread.

### Cooperative yield between isolates on the same thread

JS executing inside isolate A may `await` (e.g. via an async op), suspending
its driver task. The thread's `LocalSet` then polls another task — possibly
isolate B's driver — and that isolate makes progress. When A becomes ready
the LocalSet resumes A's task.

This is safe because every entry into V8 is wrapped in an explicit
`v8::Isolate::enter() / exit()` pair around the synchronous V8 work.
**At every `.await` point the thread's V8 enter stack is empty (or topped by
an unrelated isolate)**, so whichever task wakes next is free to enter its
own isolate cleanly.

## Invariants

These invariants make N-isolates-per-thread sound. Breaking any of them
risks V8 fatal aborts (e.g. `Cannot create a handle without a HandleScope`,
or `OwnedIsolate instances must be dropped in the reverse order of creation`).

1. **No V8 work crosses an `.await`.**
   Every block that touches a V8 handle, scope, or runtime must live inside
   a single synchronous `enter() → ... → exit()` window. Futures returned
   from V8 calls register their completions through op state, not by holding
   V8 handles across yield points.

2. **All sync V8 work is wrapped in `enter()`/`exit()`.**
   `SkiInstance::load` calls `isolate.exit()` once initialization finishes,
   so a freshly loaded isolate does not claim the thread's V8 default.
   `SkiInstance::call` and `SkiInstance::drive` each `enter()` before any
   V8 access and `exit()` after, even on early return / error paths.

3. **Drop order is free.**
   `Drop for SkiInstance` re-enters the isolate so that rusty_v8's automatic
   `OwnedIsolate::Drop` (which exits + asserts `GetCurrent() == self`) sees
   a balanced stack. Instances may therefore be dropped in any order on the
   same thread.

4. **The watchdog only signals; it never enters.**
   The watchdog thread holds an `IsolateHandle` and may call
   `terminate_execution` from outside the owning thread. That is the only
   cross-thread V8 surface ski uses, and V8 itself guarantees it is safe.
   No other code path enters or otherwise touches an isolate from a
   non-owning thread.

## Non-goals

- Not a Node runtime: no `require`, no `module`, no dynamic `import`,
  no filesystem-style module resolution.
- Not "more isolates per core, fewer threads": scaling is **more threads**,
  not packing more isolates onto one thread to amortize work. The N
  isolates per thread exist because hot-swap and hash-bucket bundling
  require it, not as a performance lever.
- Not thread-safe per isolate: only the owning thread may call `call` or
  `drive` on a `SkiInstance`. Cross-thread access is rejected at compile
  time via `!Send`.
