User pass rust project path to this program.

This program analyzes Rust code that,

1. Finds `src/pages/**/mod.rs`
2. Filters only those with `pub fn handler`
3. That mod must have `Props` type defined
4. Extracts the `Props` type definition recursively
5. Converts Rust type definitions to TypeScript equivalents
6. Outputs the TypeScript definitions to stdout or a file
