Here is the English translation of the document.

---

# OCI Rust SDK Auto-generation Guide

This guide explains how to automatically generate the Rust SDK from the OCI TypeScript SDK.

## Table of Contents

- [Overview](#overview)
- [Pipeline Architecture](#pipeline-architecture)
- [Quick Start](#quick-start)
- [Adding a New Service](#adding-a-new-service)
- [Advanced Usage](#advanced-usage)
- [Troubleshooting](#troubleshooting)
- [Code Generation Rules](#code-generation-rules)

---

## Overview

The OCI Rust SDK automatically generates Rust code using Oracle's official TypeScript SDK as the source.

### Overall Process

```
TypeScript SDK → [Discovery] → [Parser] → [Generator] → Rust SDK
                    (Node.js)    (ts-morph)   (Rust/Tera)
```

### Supported Services

Currently, the following services are implemented:
- **core** - Compute, VirtualNetwork, BlockStorage (986 models)
- **audit** - Audit Service (8 models)
- **containerinstances** - Container Instances (76 models)
- **osmanagementhub** - OS Management Hub (329 models)
- **resourcesearch** - Resource Search (10 models)

---

## Pipeline Architecture

### Step 1: Service Discovery

**Tool**: `tools/discovery/discover.ts`
**Output**: `data/services-metadata.json`

Searches for all available services in the OCI TypeScript SDK and extracts metadata.

```bash
cd tools/discovery
npm run discover > ../../data/services-metadata.json
```

**Output Example**:
```json
{
  "name": "core",
  "path": "lib/core",
  "clients": ["ComputeClient", "VirtualNetworkClient"],
  "modelCount": 678,
  "priority": 1
}
```

### Step 2: TypeScript Parsing

**Tool**: `tools/parser/parse-models.ts`
**Output**: `data/parsed/{service}-models.json`

Analyzes the TypeScript AST to extract interfaces, enums, and type information.

```bash
cd tools/parser
npx ts-node parse-models.ts --service=core > ../../data/parsed/core-models.json
```

**Key Features**:
- Interface parsing (fields, types, required/optional)
- Enum parsing (variants, values)
- Namespace enum handling (`Shape.LifecycleState` → `ShapeLifecycleState`)
- **Rust keyword automatic escaping** (`type` → `r#type`)
- Preservation of JSDoc documentation

### Step 3: Rust Code Generation

**Tool**: `tools/generator/`
**Output**: `src/{service}/models/*.rs`

Generates Rust code using the Tera template engine.

```bash
cd tools/generator
cargo run --release -- \
  --input ../../data/parsed/core-models.json \
  --output ../../src/core/models
```

**Generated Files**:
- `{model_name}.rs` - Rust file for each model
- `mod.rs` - Module exports (requires manual creation)

### Step 4: Validation & Formatting

Runs automatically:
- `cargo check` - Compilation verification
- `cargo clippy --fix --allow-dirty` - Lint auto-fix
- `cargo clippy --all-features` - Final verification

---

## Quick Start

### Generate Full Service

```bash
# Single service
./generate-sdk.sh audit

# Multiple services
./generate-sdk.sh core,audit,containerinstances

# Check service list
./generate-sdk.sh --list
```

### Verify After Generation

```bash
# Check compilation for a specific service
cargo check --features audit

# Check all services
cargo check --all-features

# Clippy verification
cargo clippy --all-features
```

---

## Adding a New Service

### Step 1: Check TypeScript SDK

```bash
# Verify if the service exists
ls oci-typescript-sdk/lib/
```

Service directory structure:
```
oci-typescript-sdk/lib/{service}/
├── lib/
│   └── model/
│       ├── *.ts (Model files)
│       └── index.ts
└── index.ts
```

### Step 2: Check Service Naming Rules

**Important**: The Rust SDK uses names without underscores.

| TypeScript | Rust Feature | Module Path |
|------------|--------------|-----------|
| `container-instances` | `containerinstances` | `src/containerinstances/` |
| `os-management-hub` | `osmanagementhub` | `src/osmanagementhub/` |
| `resource-search` | `resourcesearch` | `src/resourcesearch/` |

### Step 3: Generate SDK

```bash
# 1. Generate service
./generate-sdk.sh identity

# 2. Add feature to Cargo.toml
# (generate-sdk.sh adds it automatically, but double-check is required)
```

Add the following to `Cargo.toml`:
```toml
[features]
identity = []
```

### Step 4: Register Module in src/lib.rs

```rust
#[cfg(feature = "identity")]
pub mod identity;
```

### Step 5: Create mod.rs

**Important**: Since the generator does not create `mod.rs` automatically, manual creation is required.

```bash
# Auto-generation script
ls src/identity/models/*.rs | grep -v mod.rs | \
  xargs -I {} basename {} .rs | \
  awk '{print "pub mod "$1";\npub use "$1"::*;"}' > src/identity/models/mod.rs
```

Or manually create `src/identity/models/mod.rs`:
```rust
pub mod user;
pub use user::*;
pub mod group;
pub use group::*;
// ... repeat for all models
```

### Step 6: Validation

```bash
# Check compilation
cargo check --features identity

# Check Clippy
cargo clippy --features identity

# Test (if any)
cargo test --features identity
```

---

## Advanced Usage

### Generate Specific Models Only

```bash
cd tools/generator
cargo run --release -- \
  --input ../../data/parsed/core-models.json \
  --output ../../src/core/models \
  --limit 10  # Only the first 10 models
```

### Dry-run Mode

```bash
cargo run --release -- \
  --input ../../data/parsed/audit-models.json \
  --output ../../src/audit/models \
  --dry-run  # Preview only, does not write files
```

### Run Parser Manually

```bash
cd tools/parser

# Parse service
npx ts-node parse-models.ts --service=identity > ../../data/parsed/identity-models.json

# Check results
cat ../../data/parsed/identity-models.json | head -50
```

### Run Generator Manually

```bash
cd tools/generator

# Build
cargo build --release

# Run
./target/release/oci-gen \
  --input ../../data/parsed/audit-models.json \
  --output ../../src/audit/models
```

---

## Troubleshooting

### Issue 1: Parser Compilation Error

**Symptom**:
```
error TS1343: The 'import.meta' meta-property is only allowed...
```

**Solution**:
Check `tools/parser/tsconfig.json`:
```json
{
  "compilerOptions": {
    "module": "esnext",
    "moduleResolution": "node"
  }
}
```

### Issue 2: Rust Keyword Error

**Symptom**:
```rust
error: expected identifier, found keyword `type`
```

**Cause**: The TypeScript `type` field conflicts with a Rust keyword.

**Solution**: This is already handled automatically. If an error occurs:
1. Check the `RUST_KEYWORDS` array in `tools/parser/parse-models.ts`.
2. Ensure the `escapeRustKeyword()` function is being called correctly.

### Issue 3: Missing mod.rs

**Symptom**:
```
error[E0583]: file not found for module `models`
```

**Solution**:
```bash
ls src/{service}/models/*.rs | grep -v mod.rs | \
  xargs -I {} basename {} .rs | \
  awk '{print "pub mod "$1";\npub use "$1"::*;"}' > src/{service}/models/mod.rs
```

### Issue 4: Namespace Enum Error

**Symptom**:
```rust
// Incorrect generation
pub type: Vec<Shape.LifecycleState>  // Cannot use dot (.)
```

**Solution**: The parser handles this automatically:
- `Shape.LifecycleState` → `ShapeLifecycleState`
- Generated as a separate enum file: `shape_lifecycle_state.rs`

### Issue 5: Clippy Warnings

**Symptom**:
```
warning: this function has too many arguments
```

**Solution**:
```bash
# Auto-fix
cargo clippy --fix --allow-dirty --features {service}

# Or generate-sdk.sh runs this automatically
```

---

## Code Generation Rules

### Builder Pattern (Complies with CLAUDE.md)

#### With Required Fields

```rust
// Structure for required fields
pub struct LaunchInstanceDetailsRequired {
    pub compartment_id: String,
    pub shape: String,
}

pub struct LaunchInstanceDetails {
    // Required fields
    pub compartment_id: String,
    pub shape: String,
    // Optional fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

impl LaunchInstanceDetails {
    // Constructor receiving required struct
    pub fn new(required: LaunchInstanceDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,
            shape: required.shape,
            display_name: None,
        }
    }

    // set_* methods for all fields
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    // with_* methods for optional fields (unwrapping Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }
}
```

#### Without Required Fields

```rust
pub struct Configuration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retention_period_days: Option<i64>,
}

impl Configuration {
    // Constructor without arguments
    pub fn new() -> Self {
        Self {
            retention_period_days: None,
        }
    }

    pub fn set_retention_period_days(mut self, value: Option<i64>) -> Self {
        self.retention_period_days = value;
        self
    }

    pub fn with_retention_period_days(mut self, value: i64) -> Self {
        self.retention_period_days = Some(value);
        self
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new()
    }
}
```

### Enum Generation

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleState {
    #[serde(rename = "CREATING")]
    Creating,

    #[serde(rename = "ACTIVE")]
    Active,

    #[serde(rename = "DELETING")]
    Deleting,

    /// Unknown value for forward compatibility
    #[serde(other)]
    UnknownValue,
}
```

### Rust Keyword Handling

If a TypeScript reserved word is a Rust keyword:

```rust
pub struct SomeModel {
    // TypeScript: { type: string }
    // Rust: r# prefix + serde rename
    #[serde(rename = "type")]
    pub r#type: String,
}

impl SomeModel {
    // Method name without r#
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
```

### Type Mapping

| TypeScript | Rust | Import |
|------------|------|--------|
| `string` | `String` | - |
| `number` | `i64` | - |
| `boolean` | `bool` | - |
| `Date` | `DateTime<Utc>` | `chrono` |
| `any` | `serde_json::Value` | `serde_json` |
| `Array<T>` | `Vec<T>` | - |
| `{ [key: string]: V }` | `HashMap<String, V>` | `std::collections` |
| `string \| null` | `Option<String>` | - |

---

## Expanding the Generation Pipeline

### Adding New Types

Modify `tools/generator/src/type_mapper.rs`:

```rust
pub fn map_type(&self, ts_type: &str, is_optional: bool) -> (String, bool, bool) {
    let (base_type, needs_hashmap, needs_datetime) = match ts_type {
        "string" => ("String".to_string(), false, false),
        // Add new type
        "BigInt" => ("i128".to_string(), false, false),
        // ...
    };

    if is_optional {
        (format!("Option<{}>", base_type), needs_hashmap, needs_datetime)
    } else {
        (base_type, needs_hashmap, needs_datetime)
    }
}
```

### Customizing Templates

Modify `tools/generator/templates/model.rs.tera`:

```rust
// Add custom derive
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
                                               // ^^^^^^^^ Added

// Add custom attribute
{% if is_cached_model %}
#[cached]
{% endif %}
pub struct {{ name }} {
```

---

## Performance Optimization

### Parallel Generation

Generate multiple services simultaneously:

```bash
# Generate multiple services in background
./generate-sdk.sh core &
./generate-sdk.sh identity &
./generate-sdk.sh objectstorage &
wait

# Verify after all tasks are done
cargo check --all-features
```

### Incremental Generation

Skip already generated services:

```bash
# Backup existing models
mv src/core/models src/core/models.backup

# Generate anew
./generate-sdk.sh core

# Compare
diff -r src/core/models.backup src/core/models
```

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Generate SDK

on:
  push:
    paths:
      - 'oci-typescript-sdk/**'

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
        with:
          submodules: recursive

      - name: Setup Node.js
        uses: actions/setup-node@v2
        with:
          node-version: '18'

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Generate SDK
        run: ./generate-sdk.sh core,audit,identity

      - name: Run Tests
        run: cargo test --all-features

      - name: Create PR
        uses: peter-evans/create-pull-request@v4
        with:
          title: 'Auto-generated SDK updates'
```

---

## References

- **CLAUDE.md** - Builder pattern rules
- **tools/README.md** - Detailed tools documentation
- **GENERATION_REPORT.md** - Generation result report
- **VALIDATION_REPORT.md** - Validation results

---

## Contribution

Improve the SDK generation pipeline:

1.  **Parser Improvements**: `tools/parser/parse-models.ts`
    - Support new TypeScript patterns
    - Improve documentation extraction

2.  **Generator Improvements**: `tools/generator/src/`
    - Generate new Rust patterns
    - Expand type mappings

3.  **Template Improvements**: `tools/generator/templates/`
    - Improve code quality
    - Improve documentation formatting

---

**Last Updated**: 2025-12-24
**Version**: 0.3.0
