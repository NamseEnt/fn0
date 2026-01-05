# Claude Code Guidelines for Forte

## Code Style

- **No explanatory comments in code**: Do not leave explanatory comments in the source code. TODO comments are allowed.

## Project Context

- Forte is a fullstack framework (Rust backend + TypeScript/React frontend)
- Uses `forte-rs-to-ts` for Rust → TypeScript type generation
- Backend compiles to WASM (wasm32-wasip2 target is set in .cargo/config.toml)

## Dev Cycle

1. implement
2. make tests and test implemented
3. run clippy and fix
4. run cargo fmt and fix
