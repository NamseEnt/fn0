# OCI TypeScript SDK to Rust SDK Porting Tools

This directory contains the toolchain for porting the official Oracle Cloud Infrastructure (OCI) TypeScript SDK to the Rust SDK.

## Overview

The porting pipeline consists of three main components:

1. **Discovery** (Node.js) - Scans the TypeScript SDK to identify services
2. **Parser** (Node.js + ts-morph) - Parses TypeScript models using AST
3. **Generator** (Rust + Tera) - Generates Rust code following CLAUDE.md patterns

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│            OCI SDK Porting Pipeline (Hybrid)                 │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  [1] discovery/discover.ts                                   │
│      → Scans oci-typescript-sdk/lib/                         │
│      → Identifies services and clients                        │
│      → Output: data/services-metadata.json                   │
│                                                               │
│  [2] parser/parse-models.ts                                  │
│      → Parses TypeScript interfaces with ts-morph            │
│      → Extracts field metadata (required/optional)           │
│      → Output: data/parsed/{service}-models.json             │
│                                                               │
│  [3] generator/ (Rust)                                       │
│      → Maps TypeScript types → Rust types                    │
│      → Generates code with Tera templates                    │
│      → Follows CLAUDE.md builder patterns                    │
│      → Output: src/{service}/models/*.rs                     │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites

- Node.js 18+ and npm
- Rust 1.70+ and cargo
- The oci-typescript-sdk cloned in the parent directory

```bash
# Clone TypeScript SDK (if not already present)
cd /home/namse/fn0/oci-rust-sdk
git clone https://github.com/oracle/oci-typescript-sdk.git
```

### One-Command Generation

Generate models for the `core` service (default):

```bash
./generate-sdk.sh
```

Generate multiple services:

```bash
./generate-sdk.sh core,identity,objectstorage
```

## Manual Usage

### 1. Service Discovery

```bash
cd tools/discovery
npm install
npm run discover > ../../data/services-metadata.json
```

**Output**: Lists all 161 services with metadata (clients, model counts, priority)

### 2. Parse TypeScript Models

```bash
cd tools/parser
npm install
npx ts-node parse-models.ts --service=core > ../../data/parsed/core-models.json
```

**Output**: JSON with parsed models (interfaces, enums, fields)

**Example parsed model**:
```json
{
  "name": "LaunchInstanceDetails",
  "fileName": "launch-instance-details",
  "kind": "interface",
  "fields": [
    {
      "name": "compartmentId",
      "rustName": "compartment_id",
      "tsType": "string",
      "isRequired": true,
      "documentation": "The OCID of the compartment"
    }
  ]
}
```

### 3. Generate Rust Code

```bash
cd tools/generator
cargo build --release
cargo run --release -- \
  --input ../../data/parsed/core-models.json \
  --output ../../src/core/models
```

**Options**:
- `--input`: Path to parsed JSON file
- `--output`: Directory for generated Rust files
- `--limit N`: Only generate first N models (for testing)
- `--dry-run`: Don't write files, just preview

**Generated code follows CLAUDE.md patterns**:
- ✅ `{Model}Required` struct for required fields
- ✅ `new(required: {Model}Required)` constructor
- ✅ `set_{field}()` methods for ALL fields
- ✅ `with_{field}()` methods for optional fields only
- ✅ Proper serde annotations
- ✅ Documentation preserved

## Directory Structure

```
tools/
├── README.md              # This file
├── discovery/             # Service discovery tool (Node.js)
│   ├── package.json
│   ├── tsconfig.json
│   └── discover.ts
├── parser/                # TypeScript AST parser (Node.js)
│   ├── package.json
│   ├── tsconfig.json
│   └── parse-models.ts
└── generator/             # Rust code generator
    ├── Cargo.toml
    ├── src/
    │   ├── main.rs        # CLI entry point
    │   ├── models.rs      # Data structures
    │   ├── type_mapper.rs # TS → Rust type mapping
    │   └── codegen/       # Code generation logic
    │       └── mod.rs
    └── templates/         # Tera templates
        ├── model.rs.tera  # Struct template
        └── enum.rs.tera   # Enum template
```

## Type Mapping

| TypeScript | Rust | Notes |
|------------|------|-------|
| `string` | `String` | |
| `boolean` | `bool` | |
| `number` | `i64` | Default (safe for large numbers) |
| `Date` | `chrono::DateTime<Utc>` | |
| `any` | `serde_json::Value` | |
| `Array<T>` | `Vec<T>` | Recursive mapping |
| `{ [key: string]: V }` | `HashMap<String, V>` | |
| `Foo \| Bar` | Enum or first type | Polymorphic types |

## Examples

### Example 1: Generate Audit Service

```bash
# 1. Parse
cd tools/parser
npx ts-node parse-models.ts --service=audit > ../../data/parsed/audit-models.json

# 2. Generate
cd ../generator
cargo run --release -- \
  --input ../../data/parsed/audit-models.json \
  --output ../../src/audit/models
```

### Example 2: Test with Limited Models

```bash
# Generate only 10 models to test
cargo run --release -- \
  --input ../../data/parsed/core-models.json \
  --output /tmp/test-gen \
  --limit 10
```

### Example 3: Dry Run

```bash
# See what would be generated without writing files
cargo run --release -- \
  --input ../../data/parsed/core-models.json \
  --output ../../src/core/models \
  --dry-run
```

## Customization

### Adding New Type Mappings

Edit `tools/generator/src/type_mapper.rs`:

```rust
pub fn map_base_type(&self, ts_type: &str) -> (String, bool, bool) {
    match ts_type {
        "string" => return ("String".to_string(), false, false),
        "MyCustomType" => return ("CustomRustType".to_string(), false, false),
        // ... add more mappings
        _ => {}
    }
    // ...
}
```

### Modifying Generated Code

Edit Tera templates in `tools/generator/templates/`:

- `model.rs.tera` - Struct generation
- `enum.rs.tera` - Enum generation

Templates use Jinja2-like syntax:
```rust
pub struct {{ name }} {
{%- for field in all_fields %}
    pub {{ field.rust_name }}: {{ field.rust_type }},
{% endfor -%}
}
```

## Troubleshooting

### Issue: "TypeScript SDK not found"

**Solution**: Clone the TypeScript SDK in the parent directory:
```bash
cd /home/namse/fn0/oci-rust-sdk
git clone https://github.com/oracle/oci-typescript-sdk.git
```

### Issue: Parser errors

**Solution**: Ensure Node.js dependencies are installed:
```bash
cd tools/parser && npm install
cd ../discovery && npm install
```

### Issue: Generator compilation errors

**Solution**: Check Rust version and rebuild:
```bash
rustc --version  # Should be 1.70+
cd tools/generator
cargo clean
cargo build --release
```

### Issue: Generated code doesn't compile

**Solution**: Some complex types may need manual adjustment. Check:
1. Discriminated unions (polymorphic types)
2. Circular references
3. Complex nested generics

## Performance

**Core Service** (679 models):
- Parsing: ~15 seconds
- Generation: ~5 seconds
- Total: ~20 seconds

**All Services** (16,000+ models):
- Estimated: ~10-15 minutes

## Next Steps

After running the porting script:

1. **Format code**: `cargo fmt --all`
2. **Check compilation**: `cargo check --features core`
3. **Review generated code**: Check `src/{service}/models/`
4. **Module reorganization**: Follow the plan to merge compute/virtual_network into core
5. **Update Cargo.toml**: Add new service features

## Contributing

When modifying the porting tools:

1. Test with a small service first (e.g., `audit` with 8 models)
2. Validate generated code compiles
3. Check CLAUDE.md pattern compliance
4. Update this README if adding new features

## References

- [CLAUDE.md](/home/namse/fn0/oci-rust-sdk/CLAUDE.md) - Builder pattern specification
- [oci-typescript-sdk](https://github.com/oracle/oci-typescript-sdk) - Source SDK
- [Implementation Plan](/home/namse/.claude/plans/composed-dancing-bee.md) - Detailed design doc
