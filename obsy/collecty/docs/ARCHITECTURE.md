# collecty architecture

A per-machine OTLP collector written in Rust. It takes exports over OTLP/HTTP,
writes them zstd-compressed to an append-only disk queue, and ships them to
signy a segment at a time.

This document records what the collector *is* and why each decision was made.
There are no comments in the source, so every "why" that would have been one
lives here. [`CONFIGURATION.md`](CONFIGURATION.md) is the knob-by-knob reference.

## Decided choices

| Item | Decision |
|---|---|
| Deployment | One process per machine or one sidecar per pod; both are supported and neither is assumed |
| Built-in sources | Optional Linux host metrics and systemd journal inputs; disabled by default |
| Ingest protocol | **OTLP/HTTP**, protobuf and uncompressed, all three signals. `POST /v1/logs`, `/v1/traces`, `/v1/metrics` on a TCP port, 4318 by default |
| Payload handling | **Never decoded.** The bytes that arrive are the bytes that are stored and the bytes that are sent |
| Acknowledgement | After the record enters the open segment's compressor, before it is on the device. See "What an acknowledgement means" |
| Queue layout | **One queue per signal**, each numbering its segments from one. A segment holds one signal's exports and no others |
| Durability | **The segment is the unit.** One zstd stream a segment, `fsync`ed as it closes, and there is no `fsync` before that. **How far signy has got is not written down** — a segment it answered for is unlinked, so what is on disk is what is still owed |
| When the queue is full | **Drop the oldest segment**, whichever signal it belongs to. The application is never refused for lack of disk |
| Delivery | One closed segment in flight at a time, oldest first across the three signals. A resend after a crash is **suppressed by signy**, not duplicated — see "Sending twice" |
| Transport to signy | `POST /signy/api/v1/collect` with `Content-Encoding: zstd`, one route for all three signals, each request naming its own |
| Sender identity | Random 16 bytes made with the queue directory, in a file of their own. Not configurable, and not the hostname — it names the queue, not the machine |
| What a request carries | **One closed segment, from its first record.** Never part of one, and never the segment still being written |
| Tenancy | **None for what it forwards.** The tenant travels inside the payload as the `tenant.id` resource attribute (obsy issue #9), so a collector that does not decode has nothing to do. Collecty-generated metrics and enabled host-source telemetry use `COLLECTY_GENERATED_TELEMETRY_TENANT_ID` — see below |
| Self-observation | `collecty_*` metrics and enabled host-source telemetry are encoded as OTLP and pushed through collecty's own queue, plus a periodic summary on stderr. `COLLECTY_GENERATED_TELEMETRY_TENANT_ID` names every export collecty builds; unset means none of those exports are built |
| Format versioning | **None.** Nothing on disk is versioned. A queue written by another build is deleted, not migrated |
| Transport security | The ingest listener is plain HTTP with no authentication and remains inside its trust boundary. The outbound hop supports HTTP for trusted private networks or HTTPS with normal certificate and hostname verification; optional Cloudflare Access service-token credentials are sent only on that hop |

## The property the whole design rests on

Two independent formats both survive plain concatenation, and collecty is built
out of that coincidence.

**OTLP export requests concatenate.** `ExportLogsServiceRequest` carries exactly
one field, `repeated ResourceLogs resource_logs = 1`; the trace and metric
requests have the same shape. Protobuf's merge rule says parsing two serialized
messages back to back yields one message whose repeated fields are the
concatenation. So joining the serialized bytes of N exports **is** the merged
export — no decode, no re-encode, no field walking.

**zstd compresses a stream, not a message.** A segment's records go into one
encoder, one after another, and what the file holds when the segment closes is a
single stream that decompresses to all of them joined. The second export costs
almost nothing once the first has taught the compressor what this machine's data
looks like.

Only the first half is signal-specific: logs merge with logs, not with spans.
That is why a queue is per signal — a segment holds one, and the request names
it. Inside the segment each export goes in **behind a four byte length**, so
what signy receives is a sequence of records rather than one anonymous run of
bytes. Splitting them apart again is a walk over the plaintext with no decoding
in it; joining the payloads is then the same free merge as before.

```
export₁ (logs) ──encode──▶ [4]bytes₁ ─┐
export₂ (logs) ──encode──▶ [4]bytes₂ ─┼─▶ one zstd stream ─▶ logs/…001.seg
export₃ (logs) ──encode──▶ [4]bytes₃ ─┘                       (the body sent)
                                                        │
            signy: one decompress ──────────────────────┴─▶ [4]bytes₁[4]bytes₂[4]bytes₃
                   one walk, one decode ──────────────────▶ logs: bytes₁‖bytes₂‖bytes₃
```

collecty compresses each export once, on arrival, and never touches it again.
It does not decode on the way in, does not merge structurally, and does not
recompress on the way out. The file **is** the request body, byte for byte, so
sending a segment is a read and a socket write with nothing in between.

Both halves are pinned by tests in `src/wire.rs`
(`one_stream_over_many_records_decompresses_to_all_of_them`,
`concatenated_export_requests_decode_as_one_merged_request`,
`a_batch_survives_both_layers_at_once`), and the whole path is pinned end to end
by signy's `a_batch_per_signal_lands_in_the_store_its_request_names`.

## Receiving

### The supported wire, and it is the only one

**OTLP over HTTP/1.1, protobuf, uncompressed.** `POST /v1/logs`,
`POST /v1/traces`, `POST /v1/metrics`, `Content-Type: application/x-protobuf`,
no `Content-Encoding`.

This is the whole stack's ingest wire, not just collecty's. signy's own OTLP
push routes and its OTLP gRPC listener were removed (obsy, 2026-08-28): its one
write route takes a collecty batch and nothing else, so an application that
cannot reach collecty cannot reach signy either. Two consequences, both
exporter configuration:

- **OTLP JSON is not accepted anywhere.** collecty refuses it below, and there
  is no longer a second endpoint that decoded it.
- **OTLP gRPC is not accepted anywhere.** An SDK or Collector exporting over
  gRPC has to be pointed at this port over HTTP instead. Adding a gRPC receiver
  here is possible and is deliberately not done: it would be a second framing
  to keep byte-identical with the first, and the passthrough property below is
  the design.

### How it works

An HTTP/1.1 server on a TCP port, serving the three OTLP/HTTP export paths.
The request body **is** the serialized export request, so there is nothing to
decode and nothing to unwrap: the bytes hyper hands over are the bytes that go
into the queue. An empty body is a valid `ExportLogsServiceResponse` with no
`partial_success`, which is what a successful export answers, so a response
costs zero bytes and zero encoding work.

Because of this, `prost` and `opentelemetry-proto` are **not** on the receive
path at all. They appear at runtime only to *encode* collecty-generated telemetry
(see "Self-observation"), and in tests to prove the concatenation property.

Two things are refused rather than stored, both because the queue's whole
design rests on serialized export requests concatenating into one merged
request:

- **The JSON encoding**, with `415`. Concatenation is a protobuf property.
  Two JSON documents joined end to end are not a document.
- **A compressed body**, with `415`. Decompressing would make the bytes that
  arrive different from the bytes that are stored, which is the property the
  passthrough is. A client that compresses must be told, not silently
  half-served.

An unknown path is `404` and a non-`POST` is `405`, rather than either being
answered for: a collector that accepts an export nobody can name invites a
client to believe it landed somewhere.

A refusal's body is plain text naming the reason, not the `google.rpc.Status`
protobuf the OTLP specification suggests. Encoding one would put prost back on
the receive path to say something no client acts on beyond its status code.

Two ceilings guard memory:

- **Per request.** A declared `Content-Length` over the ceiling is refused with
  `413` *before* a byte of the body is read; a request that declares nothing is
  cut off at the ceiling while reading. `Intake::accept` keeps the same check
  for callers that do not arrive over HTTP — collecty queues its generated telemetry
  through it.
- **In flight.** A `Semaphore` whose permits are bytes. Each request acquires its
  own length and holds it until the record is on disk, so the memory a burst can
  reach is a declared number rather than a product of concurrency and size.

The append runs on a blocking thread, and compression happens inside it: the
open segment's encoder is one piece of state, so the appends that feed it are
serialised behind the queue's lock. That is the price of compressing across a
segment rather than a record, and it buys back more than it costs at small
export sizes, where per-record framing spent most of its time starting a new
compressor. Compression is still the only meaningful CPU this process spends.

## What an acknowledgement means

**A `200` means the record is in the open segment's compressor, not that it is
on the device or even in the kernel.** The encoder emits when a block fills, so
`write(2)` happens in blocks and `fsync` happens once, when the segment closes.

This was chosen over acknowledging later, and the reason is that it is the
option that leaves the **least** memory held anywhere:

- Acknowledging after `fsync` delays every response by a device sync. The
  application's exporter queue grows by exactly that delay, and collecty holds
  the in-flight request for the same span. Both sides pay.
- Acknowledging from an in-memory channel is no faster in practice and moves the
  bytes from the application's heap to collecty's — the same bytes, a different
  owner, and a channel-sized ceiling on top.
- Acknowledging on the way into the compressor is the only one where the bytes
  leave user memory as themselves. What is held instead is the encoder's window,
  which is one buffer for the whole segment however many records went through
  it, and what reaches the page cache is reclaimable and charged to nobody's RSS.

What that costs, by how the process ends:

- **A clean stop** loses nothing. `SIGTERM` closes the open segment on the way
  out, which is the same `fsync` a full one gets.
- **A crash** loses what the encoder had not written yet — a block's worth at
  most. The kernel still holds the blocks that had reached it, and recovery
  reads them back.
- **A power cut** loses the open segment, because nothing in it was `fsync`ed.
  `COLLECTY_SEGMENT_MAX_AGE` (1 s) is what bounds that.

The middle one is a real step back from acknowledging after `write(2)`, which is
what per-record framing allowed: there a process crash lost nothing at all. It
is the price of one stream a segment.

Flushing the encoder would narrow that window, and where to flush was the
decision this made rather than avoided. Per record is the only setting that
closes it completely, and it is the one that gives the ratio away: measured at
42× down to 4.2× on the same records. Anything coarser — every *n* records, or
on a timer — costs little ratio but only moves the window rather than closing
it, in exchange for a second durability knob that would have to be explained
against the segment. So the segment is the only unit, and it is the one an
operator already has a reason to think about.

## The disk queue

One queue per signal, under `{data_dir}/queue/`.

```
identity                          20 bytes: sender id, crc32
logs/00000000000000000001.seg     one zstd stream, closed at COLLECTY_QUEUE_SEGMENT_BYTES
logs/00000000000000000002.seg
traces/00000000000000000001.seg   its own numbering, from one
metrics/00000000000000000001.seg
```

That file is the whole of what the queue writes about itself. There is no
cursor: a segment signy has answered for is unlinked on the spot, so the oldest
file that is not still being written is the next one to send.

**A segment holds one signal.** Three signals in one stream meant the
compressor learning three shapes at once and a request signy had to fan out
record by record. Apart, only exports that look alike are compressed together
and a batch goes down one ingest path. What it costs is one open segment per
signal instead of one, so a quiet host closes up to three segments per
`COLLECTY_SEGMENT_MAX_AGE` rather than one, and sends up to three requests where
it used to send one.

**A segment has no framing of its own.** The file is a zstd stream and nothing
else, and inside it the records sit back to back under the four byte length
signy reads. There is no checksum around them, because there is nowhere left
for one to be useful: a stream is read from its start whatever happens, and the
decoder is what says where the readable part ends.

**One stream per segment rather than one frame per record.** A frame a record
was self-describing on disk — an 8 byte length and crc in front of each — and it
paid for that in the only currency that matters here. Compressing across the
segment instead, measured against this repository's corpus at 23 MiB of plain
exports:

| exports of | a frame a record | one stream | | CPU |
|---|---|---|---|---|
| 512 records | 1.07 MiB | 0.88 MiB | −17% | 1.26× |
| 64 records | 1.91 MiB | 0.93 MiB | −51% | 0.65× |
| 8 records | 7.94 MiB | 0.87 MiB | −89% | 0.18× |

The small end is where a collector actually lives, and it is where a frame a
record was worst: each one started a compressor that never saw enough data to
learn anything. Compressing them together is *cheaper* there too — most of that
CPU was frame setup. It costs a quarter more CPU at the large end, where framing
was never the expensive part, and that is the one place this trade is not free.

What it cost is the acknowledgement, and that is written up above. What made it
affordable is that a segment is now what a request carries: the file is sent
verbatim, and it is never read while it is open.

**The signal is the segment, not the record.** A record used to carry a signal
tag in front of its length, which was the same answer repeated in front of every
export in the file. Now the segment holds one signal and the request names it in
`x-collecty-signal`, said once for the whole body.

**Recovery closes what a crash left open; it does not resume it.** An encoder's
state is memory, and it died with the process, so there is no appending to an
unfinished stream. On open, each signal's last segment is decompressed as far as
it goes — an unfinished stream reads to its last complete block and then says
so — cut back to the last record that arrived **whole**, and written out again
as a stream that ends properly. Then this run starts a segment of its own under
that signal's next number.

The cut has to happen here rather than at signy: a batch whose last record is
short is a `400`, and a `400` drops the segment. Cutting one record locally is
the cheaper side of that trade by the whole segment.

Only the last segment of a signal can be unfinished, because closing a segment
syncs it before the next one is created. Earlier ones are not touched, which is
what keeps a restart from reading the whole backlog.

**Corruption is signy's to find.** A frame a record could be checked locally
with a crc, and a bad one ended the segment there. There is no equivalent inside
a stream, and adding one would mean decompressing every segment on the way out
to check what the receiver is about to check anyway. So a segment goes as it is;
a stream that does not decompress is a `400`, and a `400` is a refusal, and a
refusal drops the segment rather than retrying it forever.

**Nothing records how far signy has got.** An earlier design kept the answered
segment number beside the identity, and it said nothing the directory did not:
answering unlinks the file. The only case the number changed was a crash between
signy's answer and the unlink, and there the file is simply offered again —
signy answers that one from the headers without reading it. One wasted request
against a number to keep consistent with the files.

**The identity is replaced rather than repaired.** signy holds a high-water mark
under it, so a name that outlived the segment numbering would have every segment
under that mark skipped as one already stored. A file that fails its crc yields
a new name, and the queue comes back as a sender signy has never heard of. One
identity covers all three signals: it names the queue directory, and the three
live in it together.

For the same reason **segment numbers are never reused**. The next number comes
from the highest that signal's directory ever held, not from the highest still
in it, so a segment that recovery finds empty and unlinks does not hand its
number to the one that follows.

**What orders the three signals against each other is memory.** A number places
a segment among its own signal's and nowhere else, and both the order segments
are sent in and the order they are dropped in are meant to be the order they
stopped collecting. So the queue stamps a segment as it closes it, and nothing
of that reaches the disk: at open the order is read back off the files' own
timestamps, which record the same moment. A filesystem too coarse to separate
two of them costs a pair sent out of order, which signy does not care about —
each signal's own numbering is what it reads.

**Drop-oldest works in whole segments.** Rewriting a file to remove its head is
expensive; unlinking is one syscall. The budget is the whole queue's rather than
a share per signal, so a host that only ships logs spends all of it on them; and
what it takes when the budget is crossed is the segment that has been waiting
longest, which can belong to a signal other than the one being appended to.

**Dropped records are counted in bytes, not in records.** Counting records would
mean walking every segment at startup to know how many each holds, which is a
full read of the backlog on every restart. Bytes are free and answer the question
an operator actually asks.

**Backlog and disk usage are one number.** `collecty_queue_bytes` is both: a
delivered segment is unlinked as it is answered for, so everything on disk is
still owed. The separate backlog gauge went with the byte cursor that made the
two differ.

## Built-in host sources

The network receiver remains a byte-transparent OTLP relay. Built-in sources are
the only data collecty constructs itself, and they are opt-in so a sidecar does
not request host access by accident. A source encodes an OTLP export and submits
it through the same `Intake` and per-signal queue as an application export. It
never sends directly to signy or keeps a second delivery queue.

Host metrics are Linux-only and read a bounded set of procfs and rootfs files:
CPU time, memory and swap, load, filesystem usage, disk I/O and network I/O.
`COLLECTY_HOST_METRICS_ROOT` prefixes those paths for a container with the host
filesystem mounted read-only. Cumulative values retain the host boot time as
their OTLP start time; gauges use the collection timestamp only.

The journal source follows `journalctl --output=json --follow` and maps the
timestamp, `MESSAGE`, `PRIORITY`, selected systemd fields, host identity and
`tenant.id` into OTLP log records. Unit and minimum-priority filters are source
configuration, not a general transform language. The journal reader is an
adapter so its process or library dependency can be chosen without changing
the queue contract.

Journal cursor state is tied to the queue identity. A cursor is persisted only
after the batch has been appended, every affected open segment has been closed
and `fsync`ed, and the cursor file has itself been atomically replaced and
synced. A crash before the cursor sync replays entries; it may duplicate them,
but it cannot advance the cursor beyond durable queued data. Queue overflow
still drops whole segments and is reported as explicit data loss.

`COLLECTY_GENERATED_TELEMETRY_TENANT_ID` is required whenever a built-in source is enabled. The
source-generated resource always carries that tenant, `service.name=collecty`
and the stable host identity. If a required host path, journal reader or
permission is unavailable, startup or the source health log reports the
failure rather than silently running without that source.

Source health stays bounded and low-cardinality: the self-observation export
includes cumulative `collecty_host_metrics_exports_total`,
`collecty_host_metrics_errors_total`, `collecty_journal_exports_total` and
`collecty_journal_errors_total` families. The same failures are summarized on
stderr; no unit, cursor or journal field is used as a metric label.

## Sending

One sender task, one closed segment in flight at a time, oldest first across the
three signals. The open segments are never read: they are the ones the intake is
appending to, and keeping a reader out of them is most of what makes the rest of
this simple.

**A segment closes on size or on age**, and each signal keeps its own of both.
`COLLECTY_QUEUE_SEGMENT_BYTES` (8 MiB by default) closes it on a busy host;
`COLLECTY_SEGMENT_MAX_AGE` (1 s) closes it on a quiet one, and that is the floor
on how long a record waits before it leaves the machine. A busy signal no longer
carries a quiet one's records out with it, and a quiet one no longer waits
behind a busy one. Both numbers also decide how much is re-sent when a delivery is
cut, and how much a power cut loses, because the segment is the unit of all
three.

The size is measured in **compressed** bytes that have reached the file, which
the encoder emits a block at a time, so a segment overshoots by at most the last
block before it notices.

The request carries `Content-Encoding: zstd`, `x-collecty-sender`,
`x-collecty-signal` and `x-collecty-segment`, and the body is the segment file,
byte for byte. Nothing is stripped, joined or re-encoded, because the queue has
no framing of its own left to strip.

### Sending twice

The body carries no numbers and does not need to. **A segment is sent from its
first record every time**, so signy counts as it reads and record *i* of the
body is record *i* of that segment, on every attempt.

signy remembers, per sender **and signal**, which segment it is reading and how
many of its records it has stored. A segment it already has whole is answered
from the headers with the body unread. One it has part of is counted off to
where it stopped, and only the rest is stored. The three streams are numbered
apart and arrive interleaved, so one position per sender would have each signal
walking the others' back.

That closes the window this design used to leave open. The cursor was written
after the answer arrived, so a collecty that died in between — or an answer lost
on the way back — resent what signy already had, and every record in it was
stored a second time. Logs collapsed on merge; spans and metric samples did not.

The same holds for a delivery cut halfway. Whatever reached signy is durable
with its position beside it, and the resend is counted off to there rather than
stored again — `signy_collect_skipped_records_total` is where that shows.

### How an answer is read

| Answer | Meaning | Action |
|---|---|---|
| 2xx | the segment is durable in signy; the body is `{"stored":n}` | unlink everything of that signal at or below `n` |
| 400, 413, 415, 422 | this *segment* is not acceptable | drop it and move on |
| everything else, including 401/403/429/5xx and any connection failure | signy cannot take it *right now* | retry with backoff |

The default is **retry, not drop**: only the four statuses that say "this body
is wrong" are treated as permanent, and everything else is assumed temporary so
that a passing fault cannot destroy data. Backoff is 100 ms doubling to 30 s
with jitter. A misconfigured tenant policy no longer appears here at all — see
below.

`n` is normally the segment that was just sent. It can be *higher* — signy
answered an earlier attempt collecty never heard — and then everything up to it
is unlinked at once.

A single bad *record* does not reach this table. A record signy will never
accept — an undecodable body, one past a limit — is dropped there, logged,
counted in `signy_collect_dropped_records_total`, and the segment still finishes
with a `200`. Sending it back would only have collecty guess which record it
meant, and drop it anyway.

**A tenant signy will not serve never becomes a status at all.** signy reads the
tenant off each resource inside the record, so what it answers says whether the
body arrived and nothing about whose it was: a resource naming no tenant, an
unparseable one, one signy does not serve, or one at its storage limit is dropped
and counted in `signy_ingest_dropped_resources_total`, and the rest of the record
lands. The queue being shared is why that is the right shape. One application
exporting under a tenant signy does not serve would otherwise stop every other
application's logs, spans and metrics behind it on that machine — a mistake in
one process becomes an outage for the host. It is still real data loss, and the
counter with a warning beside it is where it shows.

**signy drops on every client error it does answer.** That is wider than this
table, and the shared queue is the same reason.

What still reaches the table as a refusal is a segment that is wrong as a
*whole*: a stream that does not decompress, or framing behind it that does not
add up. The statuses this table
calls retryable stay retryable, because they are answered before any of the body
is read: a fenced or draining instance, and a gate that is behind.

### Poison isolation

Not decoding has one cost: a corrupt payload is only discovered on the far side,
and a segment that always fails would block the queue head forever.

**Halving is gone.** It used to narrow a refused batch by halves until one
record was left and drop that one. signy now reads a segment a record at a time
and drops what it will never take on its own side, so nothing that reaches the
sender as a refusal is about a record — it is the segment's stream or its
framing, and no amount of splitting finds anything. Splitting is not even
possible any more: the records are inside one compression. A refused segment is
dropped whole, counted in `collecty_segments_refused_total` and
`collecty_bytes_refused_total`, and logged at error.

That is a blunter loss than the old search, and it is bounded by
`COLLECTY_QUEUE_SEGMENT_BYTES` rather than by one record. A segment whose
framing does not add up is a bug in this process or a disk that is lying,
neither of which the old search was really for.

## Self-observation

collecty encodes its own counters and enabled host-source telemetry as OTLP
exports and pushes them through its own queue, so they take the same path as
everything else and need no listening port. `opentelemetry-proto` is used rather
than hand-written prost structs on purpose: a hand-transcribed field number
fails silently, because the test that round-trips it decodes with the same wrong
definition.

The cost is that while signy is unreachable, the metrics describing that outage
are stuck in the queue behind it. The periodic summary written to stderr exists
for exactly that window — it is the only channel that still works when nothing
else does.

## What is deliberately not built

- **A metrics endpoint.** Adding a port would mean a second surface to secure and
  operate, for numbers that already have a path.
- **Multiple in-flight batches.** Ordered delivery with one cursor is a single
  number to reason about and to recover. Throughput here is bounded by signy, not
  by the link.
- **A read or query API.** The queue is a pipe, not a store.
- **Payload transformation.** No attribute injection, no filtering, no sampling.
  All of it requires decoding, and not decoding is the point.
- **Tenancy.** See the decisions table.
- **TLS on the ingest listener.** The input side remains plain HTTP; outbound TLS is provided for the signy hop.

## Where this depends on signy

- `/signy/api/v1/collect` must exist, accept `Content-Encoding: zstd`, read the
  signal out of `x-collecty-signal`, and read the four byte record header this
  document describes.
- signy must drop a record it will never accept rather than refusing the batch
  that carries it. collecty cannot tell one record from another without decoding,
  so a refusal it cannot act on becomes a halving search it should not have to
  run — and with one queue per signal, a batch that can never be accepted blocks
  every application on the machine that ships that signal.
- signy must read each record's tenant out of the record itself, at the
  `tenant.id` resource attribute, and must not require a tenant on the request.
  collecty has one queue per signal and not one per tenant, so a batch carries
  whatever the machine's applications exported; a request-level tenant would
  make every batch a single tenant's, which is the one thing collecty cannot
  arrange without decoding.
- signy must remember, per sender **and signal**, the segment in
  `x-collecty-segment` and how far into it it has got, and answer with the last
  segment of that signal it holds whole. Without that a resend is stored twice,
  which logs collapse on merge and spans and metric samples do not.
- signy must tolerate duplicates it does not catch. Its own recovery is still
  at-least-once across a flush boundary, so a restart there can replay records
  already in parts.
