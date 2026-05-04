# forte-json: JSON Serialization

`forte-json` is a custom streaming JSON serializer and deserializer used throughout Forte for communication between the Rust backend and the TypeScript frontend.

Crate path: `forte/json`

## Key Behaviors

### Serialization (Rust → JSON)

- **Struct fields**: `snake_case` Rust field names are converted to `camelCase` JSON keys automatically. You do not need `#[serde(rename_all = "camelCase")]`.
- **`None` fields are omitted**: `Option` fields with value `None` are not included in the output at all (not serialized as `null`).
- **Enum variants**:
  - Unit variants: `{"t": "VariantName"}`
  - Newtype variants: `{"t": "VariantName", "v": <value>}`
  - Tuple variants: `{"t": "VariantName", "v": [...]}`
  - Struct variants: `{"t": "VariantName", "field1": ..., "field2": ...}` *(keys are camelCase)*

### Deserialization (JSON → Rust)

- **Object keys**: `camelCase` JSON keys are converted to `snake_case` before deserialization.
- **Enum variant objects**: A JSON object `{"t": "SomeVariant"}` is converted to the string `"some_variant"` before matching the Rust enum — the `t` key is treated as the variant discriminator in snake_case.

## API

```rust
// Deserialize
pub fn from_slice<T: DeserializeOwned>(slice: &[u8]) -> Result<T, serde_json::Error>
pub fn from_str<T: DeserializeOwned>(s: &str) -> Result<T, serde_json::Error>

// Serialize
pub fn to_vec<T: Serialize + ?Sized>(value: &T) -> Vec<u8>
pub fn to_stream<T: Serialize + ?Sized>(value: &T) -> impl Stream<Item = Bytes>
```

`to_stream` returns a chunked byte stream suitable for streaming HTTP responses.

## Example

```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MyData {
    pub user_id: u64,
    pub display_name: String,
    pub optional_field: Option<String>,
}

let data = MyData {
    user_id: 1,
    display_name: "Alice".into(),
    optional_field: None,
};

let json = forte_json::to_vec(&data);
// Produces: {"userId":1,"displayName":"Alice"}
// Note: optional_field is omitted because it's None
// Note: snake_case → camelCase conversion is automatic
```

## Chunking

The serializer writes output in 8192-byte chunks. `to_stream` returns these chunks as a `futures::Stream<Item = Bytes>`.

## Limitations

- Cannot serialize streaming request bodies as outbound HTTP bodies (use `to_vec` then send as bytes).
- Deserialization goes through `serde_json::Value` for key transformation — not the fastest path for high-throughput scenarios.
- `forte-json` does not support `#[serde(rename)]` or `#[serde(rename_all)]` annotations interacting with its automatic case conversion — the automatic conversion always applies.
