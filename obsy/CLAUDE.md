# CLAUDE.md

## Repository layout

This repository is obsy, which is the product name. Each component lives in a
directory under it.

- `signy/` — the storage and query engine. The crate root is here, so build and
  test with `cd signy && cargo test`
- `collecty/` — the collector. Takes OTLP/HTTP, compresses with zstd into an
  append-only disk queue, and forwards batches to signy. Build and test with
  `cd collecty && cargo test`

There is no Cargo workspace at the root. Each component carries its own
`Cargo.lock` and `rust-toolchain.toml` and builds separately. obsy is a name
that never appears at build or run time: binaries, crates, env prefixes, metric
families and image names all use the component's name.

## Commit messages

Short, in English. The form is `<area>: <verb> <object>`.

- keep the subject under 50 characters, and never past 72
- start lower case, no trailing period
- imperative present tense (`add`, `fix`, `remove`, `move`)
- do not chain several statements into the subject; reasons and background go
  in the body
- write a body only when it is needed, separated from the subject by a blank
  line

The area is where the code lives: `metrics`, `logs`, `traces`, `wal`,
`manifest`, `compaction`, `query`, `api`, `bench`, `ci`, `docs`.

Examples:

```
metrics: add read API routes
bench: fail closed on unknown digest class
wal: retire segments only after checkpoint
```

When a body is warranted:

```
metrics: fix series state accounting on abort

An abort that revived an evicted series dropped its state bytes,
and release past zero wrapped the gate into refusing everything.
Revive restores the bytes; release clamps at zero.
```
