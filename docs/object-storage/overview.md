# object-storage

`object-storage` (`fn0-object-storage`) is a small S3-style object store for
Forte apps. It works in both WASI components (Forte backends) and native Rust
binaries.

Each project gets its own private bucket. Application code never sees the
storage endpoint or credentials: the fn0 runtime injects only a placeholder URL
(`FN0_OBJECT_STORAGE_URL`), and the worker's object-storage hijack rewrites,
routes, and signs every request out of band — the same hooks-only model as
[doc-db](../doc-db/overview.md).

## Connecting

```rust
use object_storage::Bucket;

// Production / `forte dev`: reads the injected FN0_OBJECT_STORAGE_URL.
let bucket: Bucket = object_storage::bucket();

// In-memory (tests).
let bucket: Bucket = object_storage::memory();
```

`Bucket` is `Clone`. Share it across your handler by cloning.

## Operations

### `put`

```rust
bucket.put("avatars/42.png", png_bytes).await?;

// With an explicit Content-Type:
bucket
    .put_with_content_type("avatars/42.png", png_bytes, Some("image/png"))
    .await?;
```

`put` accepts anything that converts into `Bytes` (`Vec<u8>`, `&[u8]`,
`Bytes`, `String`, …). An existing object at the same key is overwritten.

### `get`

```rust
let data: Option<Bytes> = bucket.get("avatars/42.png").await?;
```

Returns `None` if the key does not exist.

### `head`

```rust
let meta: Option<ObjectMetadata> = bucket.head("avatars/42.png").await?;
// ObjectMetadata { size, content_type, etag }
```

Fetches metadata without downloading the body.

### `delete`

```rust
bucket.delete("avatars/42.png").await?;
```

Succeeds whether or not the object existed.

### `list` — scan by key prefix

Lists objects whose key starts with `prefix`, in ascending key order, up to
`limit` entries. Pass `after` to resume after a key.

```rust
let page = bucket.list("avatars/", None, 100).await?;
for entry in &page.entries {
    println!("{} ({} bytes)", entry.key, entry.size);
}

// Next page, if the listing was truncated:
if let Some(cursor) = &page.next_cursor {
    let next = bucket.list("avatars/", Some(cursor), 100).await?;
}
```

`ObjectList { entries: Vec<ListEntry>, next_cursor: Option<String> }`.
`next_cursor` is `Some` only when the result was truncated; pass it as the
next call's `after`.

## Errors

Every operation returns `object_storage::Result<T>`. `object_storage::Error`
is a concrete enum: `Transport`, `UnexpectedStatus { status, message }`,
`Parse`.

## Local development

`forte dev` serves object storage from the local filesystem under
`.forte/data/objects/` — no cloud credentials, no external service. The API is
identical to production, so code paths do not change between `forte dev` and
deployed apps.
