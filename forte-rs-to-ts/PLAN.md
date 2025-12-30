````markdown
# PLAN.md

## Phase 1: Module Discovery

### Step 1.1: Access HIR

- In `after_analysis` callback, get `TyCtxt<'tcx>`
- Retrieve all items via `tcx.hir().items()`

### Step 1.2: Filter Page Modules

- For each item, check if `item.kind` is `ItemKind::Mod`
- Get file path from span: `tcx.sess.source_map().span_to_filename(item.span)`
- Match pattern: path contains `src/pages` AND ends with `mod.rs`
- Collect matching module `DefId`s

### Step 1.3: Output Progress

- Print discovered module paths to stdout
- Format: `Found: src/pages/foo/mod.rs`

---

## Phase 2: Handler & Props Validation

### Step 2.1: Find Handler Function

- For each page module, get children via `tcx.module_children(def_id)`
- Search for child named `"handler"`
- Verify: `visibility == Public` AND `DefKind::Fn`
- **Error and exit** if handler not found or not public

### Step 2.2: Find Props Type

- Search for child named `"Props"`
- Accept: `DefKind::Struct`, `DefKind::Enum`, or `DefKind::TyAlias`
- Capture `DefId` of Props
- **Error and exit** if Props not found

### Step 2.3: Output Progress

- Print: `src/pages/foo/mod.rs -> Props (struct|enum|alias)`

---

## Phase 3: Type Extraction

### Step 3.1: Initialize Type Converter

- Create struct holding `TyCtxt<'tcx>`
- Add `HashSet<DefId>` for cycle detection
- Add `Vec<TsDefinition>` to accumulate type definitions

### Step 3.2: Resolve Props Type

- Call `tcx.type_of(props_def_id).instantiate_identity()`
- Begin recursive conversion from this root type

### Step 3.3: Implement `convert_type` (Primitives)

- `Bool` → `boolean`
- `Int` / `Uint` / `Float` → `number`
- `Str` → `string`
- `Char` → `string`

### Step 3.4: Implement `convert_type` (Compound)

- `&T` / `&mut T` → peel ref, convert inner
- `[T]` / `[T; N]` → `T[]`
- `(T1, T2, ...)` → `[T1, T2, ...]`

### Step 3.5: Implement `convert_type` (Standard Library)

Follow serde serialization behavior:

- `String` → `string`
- `Option<T>` → `T | null`
- `Vec<T>` → `T[]`
- `HashMap<K, V>` / `BTreeMap<K, V>` → `Record<string, V>` (serde default)
- `HashSet<T>` / `BTreeSet<T>` → `T[]`
- `Box<T>` / `Rc<T>` / `Arc<T>` → convert inner `T`

### Step 3.6: Implement `convert_type` (ADT - Struct)

- Get `AdtDef` from `ty.kind()`
- Iterate `adt_def.all_fields()`
- For each field: extract name, recursively convert type
- **Error and exit** on unconvertible field type

### Step 3.7: Implement `convert_type` (ADT - Enum)

Follow serde's default externally tagged representation:

- Unit variant `Foo` → `"Foo"`
- Tuple variant `Foo(T)` → `{ "Foo": T }`
- Struct variant `Foo { a: T }` → `{ "Foo": { a: T } }`
- Combine all variants as union type

### Step 3.8: Handle Unknown Types

- If type is not recognized (external crate, unsupported)
- **Error and exit** with message: `Unsupported type: {type_name} in {context}`

---

## Phase 4: TypeScript Generation

### Step 4.1: Define TS AST

```rust
enum TsType {
    Primitive(String),      // "string", "number", "boolean"
    Array(Box<TsType>),
    Tuple(Vec<TsType>),
    Union(Vec<TsType>),
    Object(Vec<TsField>),
    Nullable(Box<TsType>),  // T | null
    Reference(String),      // named type reference
}

struct TsField {
    name: String,
    ty: TsType,
}

struct TsDefinition {
    name: String,
    ty: TsType,
}
```
````

### Step 4.2: Implement Formatter

- Implement `Display` for `TsType` and `TsDefinition`
- Output valid TypeScript syntax
- Keep field names as-is (snake_case, matching serde default)

### Step 4.3: Output Format

For each page module, output:

```
// src/pages/foo/mod.rs
export interface Props {
    field_name: type;
}

// (additional types if needed)
interface NestedType {
    ...
}
```

---

## Phase 5: Integration

### Step 5.1: Wire Everything Together

- Phase 1 → collect page modules
- Phase 2 → validate each module, collect Props DefIds
- Phase 3 → convert each Props to TS AST
- Phase 4 → format and print to stdout

### Step 5.2: Error Behavior

All errors are fatal:

- Missing handler → exit with error message
- Missing Props → exit with error message
- Unconvertible type → exit with error message
- Include file path and context in all error messages

```

```
