# Service contract and implementation costs

The source audited for the measured investigation is
`d56382ef236542846b3bb836570566f835b2a0e3`. The instrumentation changes no
request execution semantics. Its parent `61535c8ab` introduced attribution;
the final source additionally updates one stale research-metrics assertion
from two blocking submissions per warm read to one.

The overwrite follow-up uses `4418f506b0e93ed5d4dde5590b291d38031b8bda`.
Its only source addition is `wrapper_overwrite_probe.rs`; the measured service
library, shim, and original workload driver are unchanged from the matrix SHA.

| Mechanism | Contract requirement or implementation choice | Evidence and implication |
| --- | --- | --- |
| WAL and sync commit | Required by the current durable success boundary | Both the raw harness and service shim set sync=true and disableWAL=false. This is not a newly enabled service-only durability cost. |
| Atomic multi-key mutations | Required by Transact | Raw WriteBatch provides atomic mutations but does not by itself protect condition reads from concurrent writes. |
| Conditions and protected keys | Required by conditional transactions | Condition keys and mutation keys remain locked through the user commit. Locks are acquired before revision assignment. Removing all mutation locks would change dependency ordering and correctness. |
| Stable revisions and delete state | Required by existing revision/ABA behavior and its DocDb consumer | Deleting writes a logical tombstone with a revision rather than RocksDB Delete. DocDb get_observed retains a missing revision; TrxState::register_loaded stores it and TrackedEntry::condition emits RevisionEquals. A later condition must distinguish old missing state from create/delete/recreate. |
| One shared revision-counter lock per write | Implementation choice | Serialized mode holds the metadata key through sync commit. Reserved ranges preserve protected-key ordering and restart-safe unique logical identities without one counter access per user transaction. The default remains serialized. |
| Eight read/write blocking slots shared across tenants | Admission policy | All normal blocking closures retain the permit through completion, including user lock waits and commit. Split and sixteen-slot options already exist; neither changes the service contract. |
| Per-read blocking dispatch and async handoff | Adapter execution model | RocksDB Get can perform storage IO, so offloading has a legitimate purpose. The specific per-request queueing and wakeup scheme is not an API requirement. B-link calls its versioned read handle directly from the async service. |
| Warm tenant path existence check | Removed implementation overhead | Initialized Get/Query/Scan now clone the open database handle from the registry. Cold/missing tenants retain path checks. Do not attribute current warm-read latency to the former second submission. |
| Full-value GetForUpdate on mutation-only keys | Extra logical work | The transaction reads and decodes existing state for all mutation keys, but only condition-key states are used by condition_matches. A lock-only GetForUpdate exists in pinned RocksDB 11.8.1. Preserve early lock acquisition; separately assess corruption/error behavior before changing reads. |
| Full payload copies for condition keys | Extra materialization | Current predicates use revision and missing/present state, not payload. Engine lookup is still necessary; reading just a logical header does not imply avoiding a whole storage block or the engine's value lookup. |
| Response clone and encoding for size enforcement | Common implementation overhead | Both B-link and RocksDB clone and encode the response to check its size. This is not unique to RocksDB and is not separately attributed by this experiment. |
| Read context allocation while metrics are OFF | Existing adapter overhead | execute_request creates an Arc/read task-local context independently of the metrics option. This investigation retains it. Its isolated cost is not measured. |

Relevant source paths:

- `dodb/crates/dodb-rocksdb/src/lib.rs`: service configuration, run_blocking,
  execute_request, read_database, transact_with_hooks, condition_matches,
  encode_state/decode_state, and enforce_response_budget.
- `dodb/crates/dodb-rocksdb/shim/transaction_shim.cc`: TransactionDB Open,
  BeginTransaction sync policy, GetForUpdate and value copies.
- `dodb/crates/dodb-server/src/lib.rs`: BlinkTenantService::execute_request,
  grouped transaction coordinator, ensure_synthesized_response_budget.
- `doc-db/src/dodb.rs`: get_observed and observed_document retain missing-state
  revisions. Its ordinary get returns None for missing state, but that does not
  describe the transaction observation path. Transact discards the successful
  commit identity at this adapter boundary.
- `doc-db/src/trx.rs`: register_loaded and TrackedEntry::condition carry
  observed revisions, including missing state, into RevisionEquals.
- `dodb/experiments/mixed-harness/harness/rocksdb-bench/shim/rocksdb_shim.cc`:
  raw DB Open, synced WriteBatch, direct Get and iterator calls.
- Pinned RocksDB source
  `abeebd9630f11bd08c28b7bd43c7bdfc62050654`,
  `include/rocksdb/utilities/transaction.h`: GetForUpdate with a null value
  output retains key protection without reading the value.

Reference: [RocksDB 11.8.1 transaction API](https://github.com/facebook/rocksdb/blob/v11.8.1/include/rocksdb/utilities/transaction.h)
and [transaction guarantees](https://github.com/facebook/rocksdb/wiki/Transactions).

## Workload coverage discovered during measurement

The existing service workload's mutation_key generates new keys for Put and
conditional transaction mutations. Some Delete operations use the first 500
seed keys; subsequent deletes can read only a tombstone. Conditions usually
reference separate seeded keys. Consequently, the small mutation-only copy
mean includes many misses and is not the cost of repeatedly copying an
existing 512-byte value. The raw mixed harness instead chooses uniform writes
from its pre-seeded working set. This is another reason not to divide the two
historical throughput figures and label the ratio wrapper overhead.

The added probe preserves the original matrix and separately measures real
512-byte overwrites: sixteen disjoint existing keys, one per client, direct
DodbService requests, shared eight slots, reserved ranges, one-second warmup,
five-second admission, and drain. Three repetitions each use unconditional
Put with metrics ON, Exists plus Put in one Transact with metrics ON, and Put
with metrics OFF. Every measured key read must find exactly 522 encoded bytes
and 512 payload bytes. Each key's revision must advance, full values and the
complete sixteen-row state must match the parent-generated oracle before
and after reopen. Prefix fingerprints match within each request mode.

The probe has no contested user keys and no concurrent reads. It isolates the
covered full-value materialization workload; it does not reproduce production
hot-key contention or compare raw and wrapped calls with equivalent semantics.
