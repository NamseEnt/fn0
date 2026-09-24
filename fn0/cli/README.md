# commands

## init

inquire

- language

  - typescript

if typescript

- package manager

  - bun

- framework

  - hono


### config file

`init` command creates `fn0.toml` in project directory.

```rust
struct Config {
  language_env: LanguageEnvironment,
}
enum LanguageEnvironment {
  TypescriptBunHono,
}
```

## build

Detect environment and build wasm file.

The `build` command uses the `language_env` from `fn0.toml` to determine the correct build process.

for `TypescriptBunHono`, it just runs `bun build`.

## local

This runs the fn0 server using fn0 crate, passing dist/component.wasm as the wasm file path.

`fn0 local` supports normal semantic document access through `doc_db::database()`.
Its host service uses one in-memory database for the lifetime of the process, so
document state is lost when the local server restarts. Legacy/direct Turso access
is not configured by this command.

params

```
--port or -p optional
--wasm-path optional
```
