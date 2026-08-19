# Serialization Reference

Forte uses two different serializers depending on where data flows. This document is the canonical reference for the data format.

## Quick Summary

| Handler type | Input deserialized with | Output serialized with |
|---|---|---|
| Pages | N/A (no input body) | `forte_json` |
| APIs | `forte_json` (if you call `forte_json::from_slice`) or raw bytes | `forte_json` |
| Actions | `forte_json` | `forte_json` |
| Hooks | `forte_json` | `forte_json` |
| Queue tasks | `serde_json` (no key conversion) | N/A |
| Admin tasks | `serde_json` (no key conversion) | `serde_json` |

## `forte_json` Rules

`forte_json` is the serializer/deserializer for all action, hook, page, and API handler I/O.

### Key conversion

**Serialization (Rust → JSON):** struct field names are converted `snake_case` → `camelCase`.

```rust
pub struct User {
    pub first_name: String,    // → "firstName"
    pub last_name: String,     // → "lastName"
    pub created_at: u64,       // → "createdAt"
}
```

**Deserialization (JSON → Rust):** all object keys are converted `camelCase` → `snake_case` before deserializing. TypeScript sends `{ firstName: "Alice" }` and the Rust struct receives `first_name: "Alice"`.

### Enum variant encoding

Enum variants use a `t` discriminant field:

| Variant kind | Rust | JSON |
|---|---|---|
| Unit | `Ok` | `{"t":"Ok"}` |
| Tuple / newtype (one field) | `Ok(String)` | `{"t":"Ok","v":"..."}` |
| Struct | `Ok { message: String }` | `{"t":"Ok","message":"..."}` |

Struct variant fields are spread **flat** alongside `t` — there is no `v` wrapper. Field names are camelCase.

The TypeScript generated types use `t` as the discriminant key for `z.discriminatedUnion("t", [...])`.

### `Option::None` omission

`None` struct fields are **omitted entirely** from the JSON output — they do not appear as `null`.

```rust
pub struct Response {
    pub id: String,
    pub nickname: Option<String>,   // omitted when None, not "nickname":null
}
```

This means TypeScript optional fields should be typed `nickname?: string` (optional), not `nickname: string | null` (nullable). The generated `.props.ts` types already reflect this.

A top-level `None` value (not inside a struct field) serializes as `null`.

### API reference

```rust
use forte_sdk::forte_json;

// Serialize
let bytes: Vec<u8> = forte_json::to_vec(&value);
let stream = forte_json::to_stream(&value);   // Stream<Item=Bytes>, 8 KiB chunks

// Deserialize
let value: T = forte_json::from_slice(&bytes)?;
let value: T = forte_json::from_str(json_str)?;
```

`to_stream` is a lazy stream — use it when building a streaming HTTP response body.

## Queue Task and Admin Task Input

Queue task and admin task handlers receive input deserialized with **standard `serde_json`**, not `forte_json`. There is no camelCase → snake_case key conversion.

```rust
// Queue task input: field names must match JSON keys exactly
pub struct Input {
    pub user_id: String,   // caller must send { "user_id": "..." }
}
```

The generated `enqueue::<name>(input)` function serializes with `serde_json`, so enqueued tasks always use the correct format. Admin task input via `forte admin run --input '...'` or `--input-file` must also use snake_case keys.

Admin task output (printed to the terminal by the CLI) is also serialized with `serde_json`, so snake_case field names appear in the output.

## TypeScript Type Mapping

`forte-rs-to-ts` converts Rust types to TypeScript. The mapping:

| Rust | TypeScript |
|---|---|
| `String`, `&str` | `string` |
| `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `usize`, `isize`, `f32`, `f64` | `number` |
| `bool` | `boolean` |
| `Option<T>` | `T \| undefined` (optional in structs) |
| `Vec<T>` | `T[]` |
| `HashMap<String, V>` | `Record<string, V>` |
| `chrono::DateTime<_>` | `Date` (Zod: `z.coerce.date()`) |
| `serde_json::Value` | `unknown` (Zod: `z.json()`) |
| unit enum variant | `{ t: "VariantName" }` |
| tuple/newtype variant | `{ t: "VariantName"; v: T }` |
| struct variant | `{ t: "VariantName"; fieldName: T; ... }` |

Enums become TypeScript discriminated unions on `t`.

## Cookies

Cookie values are serialized with standard `serde_json` (not forte_json) and then HMAC-signed. Key names in cookie payloads are whatever serde produces — typically snake_case for structs.

## What Goes Through Each Path

```
Browser fetch → camelCase JSON
    ↓ forte_json::from_slice (action/hook input)
Rust handler (snake_case fields)
    ↓ forte_json::to_vec (action/hook/page/API output)
Response JSON → camelCase keys, t discriminant
    ↓ Zod validation in generated TS client
TypeScript (camelCase, discriminated union on t)
```

```
forte admin run --input '{"count": 10}'
    ↓ serde_json::from_str (admin task input)
Rust handler (snake_case fields, field names must match literally)
    ↓ serde_json::to_string (admin task output)
Terminal output (snake_case field names in JSON)
```

```
enqueue::<name>(input)
    ↓ serde_json::to_vec (queue task serialization)
Queue
    ↓ serde_json::from_slice (queue task input)
Rust handler (snake_case fields)
```
