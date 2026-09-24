# Storage format

The fixed page size is 4096 bytes. Phase 0 defines the common codecs and
Phase 1 uses them for a single-file mutable B+Tree. Numeric fields are
explicit little-endian values; Rust memory layouts are never written directly.

## Common page header

Each encoded page is exactly 4096 bytes and uses little-endian numeric fields:

| Offset | Size | Field |
| ---: | ---: | --- |
| 0 | 4 | magic `DBPG` |
| 4 | 2 | format version |
| 6 | 1 | page type (`Internal`, `Leaf`, `Overflow`, `Free`, `Test`) |
| 7 | 1 | reserved, must be zero |
| 8 | 8 | page id |
| 16 | 8 | page LSN |
| 24 | 4 | flags |
| 28 | 4 | CRC32C checksum |
| 32 | 4064 | body, currently opaque/zero-filled |

CRC32C is calculated over the entire page after zeroing bytes 28..32. The
codec validates exact length, magic, version, page type, checksum, and an
optional expected physical page id.

## Double superblock

Page 0 is slot A and page 1 is slot B. Each is a full-page explicit codec with
magic `DSBK`, version, generation, database UUID, tenant/shard identity and
epoch, page size, nullable root/free-list/high-water page references,
checkpoint LSN, and CRC32C. A nullable page reference is encoded as
`u64::MAX`.

On open, unsupported format is an explicit error. Otherwise the valid copy
with the highest generation is selected. Both invalid copies are corruption.
Generation wraparound is deliberately outside the operating assumptions.

Phase 1 initializes page 2 as the root leaf. `high_water_page_id` is the last
allocated data page, so a valid file has exactly `high_water_page_id + 1`
pages. Data pages are allocated from page 2 onward. `free_list_head` points to
a singly linked stack of pages whose page type is `Free`; a free page stores
the next page ID in its checked body format. Free pages are reused before the
high-water mark is advanced.

## B+Tree pages

The page header remains the Phase 0 header. Each page-specific body has its
own four-byte layout magic and version. Leaf and internal pages use slotted
layouts:

```text
page header
page layout magic/version and metadata
slot array growing upward
free space
variable-length records growing downward
```

Leaf slots point to records containing the canonical encoded document key, a
revision, and one of `Missing`, inline value, or overflow value metadata. Leaf
records are strictly ordered by the encoded key. Each leaf also stores a
right-leaf page ID. An empty leaf is allowed after deletion because Phase 1
does not merge or rebalance pages.

Internal slots contain a separator key and the right child page ID. The
separator is a partition boundary: keys in the left child are strictly less
than it and keys in the right child are greater than or equal to it. The
separator is initially the smallest key in the right subtree. It remains a
valid boundary when deletion leaves a child empty; this avoids requiring
delete rebalancing while preserving lookup correctness.

Overflow pages have an explicit next page, total value length, chunk length,
and chunk bytes. The current maximum value is 64 MiB and an overflow chunk
uses 4032 bytes. Values up to 512 bytes are inline when their complete leaf
record fits; otherwise the value uses an overflow chain. Replacing or
deleting a value places its old overflow pages on the free stack.

The maximum canonical encoded document key is 3992 bytes. This leaves room
for one leaf slot and the leaf record header; larger keys are rejected as
invalid input instead of being truncated.

All slotted offsets, lengths, page IDs, record states, ordering, and overflow
chains are checked on decode. A malformed page returns `Corruption` or an
explicit unsupported-format error and is never treated as an empty page.

## Phase 2 persistence boundary

An operation batch reads committed/cache pages into a private overlay. Updated
pages, allocator metadata, and the next superblock generation are prepared in
memory. In the WAL-backed path, full page images and the alternate superblock
image are appended to the separate redo-only WAL, followed by a `COMMIT` frame
and WAL `sync_data`. That sync is the durability point. Only after it succeeds
are the images published to the committed cache and dirty-page set. Data-file
flushing is later and is not a commit boundary. Phase 5 adds a formal
checkpoint that syncs the complete committed database image, advances the
alternate superblock's `checkpoint_lsn`, and then resets the WAL with an
identity `INIT` whose history starts after that LSN.

The legacy `NamseEnt/namseent` `luda-editor/new-server/bptree` implementation
is an architectural reference for page-oriented mutation, but dodb defines a
new incompatible format with checked decoding, explicit page identity,
checksums, reserved page LSNs, binary composite keys, and invariant checking.
