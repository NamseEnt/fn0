# obsy

An observability stack for a single machine, built as separate programs that
ship on their own.

| Component | Directory | What it is |
|---|---|---|
| **signy** | [`signy/`](signy/) | The storage and query engine. Takes collecty's batches on one route and answers a first-party HTTP API. Single machine, single writer, S3-compatible object storage as the source of truth, local disk as an evictable cache, and one declared memory budget divided into arenas. |
| **collecty** | [`collecty/`](collecty/) | The collector. Takes OTLP/HTTP, writes it zstd-compressed to an append-only disk queue, and ships it to signy in batches. It never decodes a payload: a segment is one zstd stream over the exports as they arrived, and the file is the request body byte for byte. |

`obsy` is the product and the name of this repository. It is deliberately not a
binary, a crate, an env prefix, a metric family or an image — nothing at build
or run time is called obsy. Each component owns its name everywhere it is
visible: signy's knobs are `SIGNY_*`, its metric families are `signy_*`, its
routes are `/signy/...`, its headers are `X-Signy-*` and its image is
`ghcr.io/namse/signy`. Image tags do not derive from the repository name, so
signy's image and collecty's stay distinct.

## How telemetry gets in

```
app ──OTLP/HTTP──▶ collecty ──▶ signy
```

**That is the only way in, and it is one way on purpose.** signy has a single
write route, `POST /signy/api/v1/collect`; its OTLP push endpoints and its OTLP
gRPC listener were removed. An engine an application can push to directly has no
queue in front of it, so a refusal it gives — draining for a restart, flush
behind, disk low — is telemetry lost unless that application happens to hold it.
collecty's append-only disk queue is the thing that holds it, and a queue only
helps when nothing can go around it.

So the supported wire, for the whole stack, is **OTLP over HTTP/1.1, protobuf,
uncompressed**, on `POST /v1/logs`, `/v1/traces` and `/v1/metrics` at collecty.
Two things follow, and they are exporter configuration rather than
limitations to work around:

- **No OTLP JSON.** collecty refuses it with `415` — it never decodes a
  payload, so it cannot re-encode one — and nothing downstream accepts it either.
- **No OTLP gRPC.** Point the exporter at collecty over HTTP.

## Layout

```
obsy/
├── .github/workflows/   one workflow per component
├── CLAUDE.md            repository conventions
├── collecty/            the collector: Cargo.toml, src, docs, benches
└── signy/               the engine: Cargo.toml, src, docs, benches, compare, deploy
```

There is no Cargo workspace at the root. Each component is self-contained under
its own directory, carries its own `Cargo.lock` and `rust-toolchain.toml`, and
builds on its own.

That was decided rather than inherited. A root workspace would share one
`Cargo.lock`, which means a dependency bumped for collecty changes what signy
builds — the opposite of components that ship on their own. It would also move
the lock out of `signy/`, breaking an image build that copies it from there.
Neither cost buys anything: the two crates share no code.

## Working on signy

```sh
cd signy
cargo test
```

The engine documents itself under [`signy/docs/`](signy/docs/):
[`ARCHITECTURE.md`](signy/docs/ARCHITECTURE.md) for what it is,
[`VISION.md`](signy/docs/VISION.md) for what it is for and what would falsify
the claim, [`QUERY_API.md`](signy/docs/QUERY_API.md) for the API contract,
[`CONFIGURATION.md`](signy/docs/CONFIGURATION.md) for every knob, and
[`DEPLOYMENT.md`](signy/docs/DEPLOYMENT.md) with
[`RUNBOOK.md`](signy/docs/RUNBOOK.md) for running it.

## Working on collecty

```sh
cd collecty
cargo test
```

The collector documents itself under [`collecty/docs/`](collecty/docs/):
[`ARCHITECTURE.md`](collecty/docs/ARCHITECTURE.md) for what it is and why each
decision was made, and
[`CONFIGURATION.md`](collecty/docs/CONFIGURATION.md) for every knob and what
raising it costs.

collecty can optionally collect a bounded set of Linux host metrics and the
systemd journal in a per-machine deployment. Those sources are disabled by
default and feed the same OTLP disk queue as network exports; collecty does not
become a general-purpose pipeline engine.

collecty carries no comments in its source. Every "why" that would have been one
is in `docs/ARCHITECTURE.md` instead.
