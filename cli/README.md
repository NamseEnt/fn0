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
  - astro (with SSR support)

### config file

`init` command creates `fn0.toml` in project directory.

```rust
struct Config {
  language_env: LanguageEnvironment,
}
enum LanguageEnvironment {
  TypescriptBunHono,
  TypescriptBunAstro,
}
```

## build

Detect environment and build wasm file.

The `build` command uses the `language_env` from `fn0.toml` to determine the correct build process.

For `TypescriptBunHono`, it runs `bun run build`.

For `TypescriptBunAstro`, it runs `bun run build` which executes:
- `astro build` (generates SSR server code and static assets)
- `bun build` (bundles server code into single file)
- `jco componentize` (converts to WASM component)

## local

This runs the fn0 server using fn0 crate.

For Astro projects, the server serves static assets (from `dist/client`) directly from the Rust host for better performance, while dynamic requests are handled by the WASM component.

params

```
--port or -p optional
```
