# forte-json

`forte-json` is a custom JSON serializer/deserializer used throughout Forte. It applies automatic case conversion and uses a specific discriminated-union format for Rust enums that matches what `forte-rs-to-ts` generates on the TypeScript side.

## Serialization (`to_vec`, `to_stream`)

```rust
use forte_json;

// To Vec<u8>
let bytes: Vec<u8> = forte_json::to_vec(&my_value);

// To a Stream of 8 KiB chunks (used for streaming HTTP responses)
let stream = forte_json::to_stream(&my_value);
```

### Struct field names

Snake_case Rust fields are serialized as camelCase JSON keys:

```rust
#[derive(Serialize)]
struct User {
    user_id: u32,
    display_name: String,
}
// → {"userId":1,"displayName":"Alice"}
```

### `Option::None` fields are omitted

Struct fields set to `None` are not written to the output at all:

```rust
#[derive(Serialize)]
struct Profile {
    name: String,
    bio: Option<String>,
}

Profile { name: "Alice".into(), bio: None }
// → {"name":"Alice"}   (bio is absent, not "bio":null)
```

### Enum variants

The format depends on the variant kind:

| Variant kind | Example | JSON output |
|---|---|---|
| Unit | `Status::Ok` | `{"t":"Ok"}` |
| Newtype | `Wrap::Value(42)` | `{"t":"Value","v":42}` |
| Tuple | `Pair::Of(1, 2)` | `{"t":"Of","v":[1,2]}` |
| Struct | `Msg::Hello { text: "hi" }` | `{"t":"Hello","text":"hi"}` |

For struct variants the fields are inlined alongside `"t"` — there is no `"v"` wrapper.

This format is what React page components receive as `props` and what `forte-rs-to-ts` generates TypeScript discriminated-union types for.

## Deserialization (`from_slice`, `from_str`)

```rust
use forte_json;

let value: MyInput = forte_json::from_slice(bytes)?;
let value: MyInput = forte_json::from_str(json_str)?;
```

These are used by the generated dispatcher to decode action and hook request bodies sent by the TypeScript frontend.

### Key conversion

CamelCase JSON keys are converted to snake_case before passing to serde:

```
{ "userId": 1 }  →  { "user_id": 1 }
```

### Unit variant deserialization

A JSON object with exactly one key `"t"` is collapsed to a plain string (snake_case of the variant name) before deserialization. This allows serde's default string format for unit enum variants to work with the `{"t":"..."}` format the frontend sends:

```json
{ "t": "SomeVariant" }  →  "some_variant"
```

serde then maps `"some_variant"` to the correct Rust variant (assuming `#[serde(rename_all = "snake_case")]` or matching names).

## Chunk size

`to_stream` emits chunks of up to 8 192 bytes. Smaller trailing data is flushed as a final chunk.

## When to use forte-json vs serde_json

| Use case | Recommended |
|---|---|
| Serialize `Props` for SSR | `forte_json::to_stream` / `forte_json::to_vec` |
| Deserialize action `Input` from browser | `forte_json::from_slice` (done automatically by the dispatcher) |
| Serialize outbound API request body | `serde_json` (no case conversion needed for external APIs) |
| Arbitrary internal JSON | `serde_json` |

The generated `route_generated.rs` uses `forte_json` automatically for all page and action payloads. You typically only call these functions directly when implementing custom serialization logic.
