# Phase F design: leaf-local packed storage

Branch `experiment/wal-v3-compact-redo`, on top of Phase E (`075403a`, code `4ead3de`). Only the in-memory representation of planned-Blink leaves changes. The page format, WAL v3 and PageDelta formats, commit and revision rules, routing, internal pages, overflow pages, checkpoints, the parallel executor and the planner are unchanged.

## F0 — leaf population probe (before any code change)

Commit `9d9927a` adds, behind the `churn-counters` feature only, one sample per lane clone: the touched leaf's entry count, key bytes and inline-value bytes. OCI, 64 writers, width 16, sync disabled, 2 lanes, 2 runs each (`raw/probe-*`):

| Distribution | touched leaves / tx | entries / leaf mean (p50 / p95 / max) | key bytes / leaf mean (p95 / max) | inline value bytes / leaf mean (p95 / max) | payload bytes / leaf mean (p50 / p95 / max) |
|---|---|---|---|---|---|
| uniform | 14.85 | 15.1 (15 / 15 / 29) | 495 (495 / 933) | 970 (960 / 1,856) | 1,464 (1,455 / 1,455 / 2,789) |
| compact | 0.12 | 14.0 (14 / 14 / 14) | 490 (491 / 491) | 896 (896 / 896) | 1,386 (1,386 / 1,387 / 1,387) |
| spread | 2.0 | 15.0 (15 / 15 / 15) | 478 (482 / 512) | 960 (960 / 960) | 1,438 (1,440 / 1,442 / 1,472) |

The benchmark's encoded keys are about 33 bytes (zero bytes in the key are escaped) and values 64 bytes.

Estimated work per touched-leaf clone (uniform):

| | Phase E (`Vec<LeafEntry>`, key and value `Arc`s) | Packed candidate |
|---|---|---|
| copy | 15 × 48-byte entries = 720 bytes | 15 × 40-byte slots = 600 bytes + 1,464 payload bytes ≈ 2.1 KB, two sequential copies |
| reference counts | 30 increments at clone + 30 decrements when the old version retires, each on a payload cache line last touched by another core | none per entry |
| allocations at clone | entry vector + high key | slot vector + byte buffer + high key |
| allocations per new entry | key `Arc` (the value `Arc` came from the planner) | none (bytes appended to the leaf buffer) |

So the candidate copies about 2 KB per touched leaf, not 4 KB. The risk is that 1.4 KB of extra sequential copy per leaf costs as much as the 60 remote count changes it replaces; that is what F1 measures.

## Ownership model

Phase E: `BlinkState.pages` and every published generation share one `Arc<BlinkPage>` per page version (E2a). A leaf page holds `Vec<LeafEntry>`; each entry owns an `Arc<[u8]>` key and an `Arc<[u8]>` inline value. Copy-on-write of a leaf (in its lane) clones the vector and increments 2 counts per entry; the retired version's drop decrements them.

Phase F: same page-level sharing (`Arc<BlinkPage>`, one count per page version, which is the coarse-grained immutable object the plan asks for). A leaf page holds `LeafEntries` (`blink/leaf.rs`):

```text
LeafEntries { slots: Vec<LeafSlot>, bytes: Vec<u8>, garbage: usize }
LeafSlot    { revision, key_offset: u32, key_length: u32,
              value: Missing | Inline { offset: u32, length: u32 }
                            | Overflow { head: PageId, length: u64 } }
```

Keys and inline values of the leaf live in `bytes`; slots hold offsets. Nothing inside a leaf is reference counted. A copy-on-write clone copies the two vectors (with a little spare capacity); dropping a retired leaf frees two buffers. There is no separate `Arc<LeafData>`: the page is already shared through `Arc<BlinkPage>`, and a second count inside it would only be bumped together with the first.

- Mutation API: `search`, `get` / `key` / `iter` / `range` (read views `LeafEntryRef { key, revision, value }`), `insert`, `replace`, `set_revision`, `split_off`, `remove` (tests). Callers never do offset arithmetic.
- A replacement with an inline value of the same length writes over the old bytes (the benchmark's updates); any other replacement appends and counts the old bytes as garbage. Garbage is dropped by the next clone (which rebuilds a compact buffer) or when it passes half of the buffer (and 256 bytes).
- Overflow values keep the `Overflow { head, length }` reference; overflow pages are unchanged.
- Encoder, decoder, `leaf_body_layout`, `leaf_fits`, `choose_leaf_split` and `ensure_sorted_leaf` run the same rules on `LeafRange` views; no encoding rule was rewritten. Decode builds the packed leaf directly (one slot vector and one byte buffer per page instead of two `Arc`s per entry).
- Published pages are only read. A writer copies a committed leaf (`BlinkPage::clone` or `Arc::make_mut` in the non-planned serial path) before changing it.
- No `unsafe`.

## Correctness proofs added

- Randomized differential (`packed_leaf_matches_reference_model_randomized`): 400 seeds × 120 operations on empty, small, and nearly full leaves (inline values, overflow references, tombstones), comparing the packed leaf with a Phase-E-style owned entry list after every insert / replace (same and different length) / revision change / remove: entries, binary-search results for present and absent keys, fit check, encoded page bytes against the independent reference encoder, decode round trip, split choice and both split halves, clone equality and clone independence. It also checks that compaction runs.
- Packed ownership: a page clone has its own key and value bytes and leaves the original's pointers and contents unchanged; a working-overlay replace and restamp leave the committed page object, its payload pointers and contents unchanged.
- Phase E's pinned-generation test is unchanged and passes; its pointer assertions now check addresses inside each pinned leaf's byte buffer.
- Phase E vs Phase F on the public API: identical results, commit LSNs, get / query / scan, reopen scan, data-file bytes and WAL bytes (`crossver/`).
