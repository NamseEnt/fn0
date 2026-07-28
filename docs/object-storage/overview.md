# object-storage

`object-storage` (`fn0-object-storage`) is a small S3-style object store for
Forte apps. It works in both WASI components (Forte backends) and native Rust
binaries.

Storage is split by access model into two namespaces, which are separate types
rather than two configurations of one:

| | Reachable by | Use for |
|---|---|---|
| `object_storage::private` | the app, or a presigned URL | anything not meant to be world-readable |
| `object_storage::public` | anyone with the URL, served from the CDN | assets embedded in HTML that outlives a signature |

Application code never sees the storage endpoint or credentials: each namespace
reads only its injected placeholder URL, and the worker's object-storage hijack
rewrites, routes, and signs every request out of band — the same hooks-only
model as [doc-db](../doc-db/overview.md).

## Connecting

```rust
use object_storage::private::PrivateBucket;

// Production / `forte dev`: reads the injected FN0_OBJECT_STORAGE_URL.
let bucket: PrivateBucket = object_storage::private::bucket();

// In-memory (tests).
let bucket: PrivateBucket = object_storage::private::memory();
```

`PrivateBucket` is `Clone`. Share it across your handler by cloning.

## Operations

### `put`

```rust
bucket.put("avatars/42.png", png_bytes).await?;

// With an explicit Content-Type:
bucket
    .put_with_content_type("avatars/42.png", png_bytes, Some("image/png"))
    .await?;
```

`put` accepts anything that converts into `object_storage::Body` — `Vec<u8>`,
`&[u8]`, `Bytes`, `String`, `&str`. An existing object at the same key is
overwritten.

Inside a WASI component a `forte_sdk::http::Body` also converts, which forwards
an incoming request body straight through without buffering it:

```rust
bucket.put_with_content_type("uploads/clip.mp4", request.into_body(), Some("video/mp4")).await?;
```

The length is then unknown until the body ends, so the upload is sent chunked
rather than with a `Content-Length`.

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

## Presigned URLs

`presigned_get_url` / `presigned_put_url` return a time-limited URL that a
browser (or any HTTP client) can use to download from or upload to an object
directly, without routing through the app.

```rust
use std::time::Duration;

let download = bucket
    .presigned_get_url("avatars/42.png", Duration::from_secs(3600))
    .await?;
// hand `download` to a browser <img src> or fetch()

let upload = bucket
    .presigned_put_url("uploads/new.bin", None, Duration::from_secs(900))
    .await?;
// browser: fetch(upload, { method: "PUT", body: file })

// bound to an upload of exactly 1 MiB — anything else is rejected
let bounded = bucket
    .presigned_put_url("uploads/new.bin", Some(1024 * 1024), Duration::from_secs(900))
    .await?;
```

`content_length` binds the URL to an upload of exactly that many bytes; `None`
accepts any size. Use it when handing an upload URL to an untrusted end user —
otherwise a leaked URL can store an object of any size against your project's
storage quota. The bound is exact rather than a maximum because SigV4 cannot
express a size range, so callers that do not know the size up front must read
it first (a browser's `File.size`) and mint the URL per upload.

The URL is signed by the worker's object-storage hijack — application code
never holds credentials. The SigV4 signature, expiry, R2 endpoint, account id
and bucket name appear in the URL; the secret access key never does. On fn0
Cloud, `expires` is capped at 5 minutes (`PRESIGN_MAX_EXPIRES_SECS = 300`);
longer requested durations are clamped, not rejected. Self-hosted deployments
have no cap.

Presigned URL minting counts against per-project quotas (100k/month,
1k/hour on the one-dollar plan). Exceeding the quota blocks minting with
HTTP 429 until the window resets; already-minted URLs stay valid until they
expire. See [limits.md](../fn0/limits.md) for full quota values.

In `forte dev` the URL points at the dev server's local object route
(`/__fn0_object_storage/…`) and does not expire.

## Local development

`forte dev` serves object storage from the local filesystem under
`.forte/data/objects/` — no cloud credentials, no external service. The API is
identical to production, so code paths do not change between `forte dev` and
deployed apps.
