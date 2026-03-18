# Forte CLI Implementation Progress

## Completed Tasks

### 1. Project Structure Refactoring ✅

CLI and server logic separation complete.

```
forte/src/
├── main.rs           # CLI entry point (clap)
├── cli/
│   ├── mod.rs        # Command definitions (dev, init, add, build)
│   ├── dev.rs        # forte dev implementation
│   └── init.rs       # forte init implementation
└── server/
    ├── mod.rs        # SSR server logic
    └── cache.rs      # SimpleCache (WASM/JS caching)
```

### 2. forte init ✅

- Project structure generation (rs/, fe/)
- Config file generation: Forte.toml, Cargo.toml, package.json, etc.
- Default page template generation (index page)
- E2E tests passing (3)

### 3. forte dev ✅

- Auto port selection (starting from 3000, next port if in use)
- `--port` option to specify exact port
- `forte-rs-to-ts` invocation (Props type generation)
- `cargo build --release` invocation (backend WASM build)
- `npm run build` invocation (frontend build)
- SSR server start and request handling
- E2E tests passing (2)

### 4. Frontend Router Auto-generation ✅

- `rs/src/pages/` directory scanning
- Only files with `pub async fn handler` registered as routes
- `fe/src/routes.generated.ts` auto-generation
- Dynamic route support (`[id]` → `:id`)
- Routing via `matchRoute()` function in `server.tsx`

### 5. forte add page ✅

- Verify current directory is a Forte project (Forte.toml check)
- Backend page generation (`rs/src/pages/<path>/mod.rs`)
- Frontend page generation (`fe/src/pages/<path>/page.tsx`)
- Nested path support (`product/detail` → `product/detail/mod.rs`)
- Dynamic route support (`product/[id]` → includes `Params` struct)
- E2E tests passing (5)

### 6. Watch Mode + Hot Swap ✅

- Using `notify` + `notify-debouncer-mini` crates
- Watching `rs/src` and `fe/src` directories
- Auto rebuild on file change (500ms debounce)
- Hot swap via cache invalidation (no server restart)
- Separate backend/frontend rebuild support

### 7. forte add action ✅

- Backend action file generation (`rs/src/actions/<path>.rs`)
- Frontend client generation (`fe/src/actions/<path>.ts`)
- Input/Output type auto-generation
- Fetch call to `/_action/<path>` endpoint
- E2E tests passing (4)

### 8. Static Asset Serving ✅

- Static file serving at `/public/*` path
- `/favicon.ico` auto-handling
- Auto MIME type detection (images, CSS, JS, fonts, etc.)
- Cache header configuration (`Cache-Control: public, max-age=3600`)
- Path traversal attack prevention
- `fe/public/` directory auto-creation (init)

### 9. forte build ✅

- Production build command
- Codegen execution (forte-rs-to-ts, routes.generated.ts)
- Backend WASM build (release mode)
- Frontend build (npm run build)
- Deployment files generated in `dist/` directory
  - `backend.wasm`
  - `server.js`
  - `public/` (static file copy)
- E2E tests passing (2)

### 10. Hydration Support ✅

Client-side React mounting implementation complete.

- `fe/src/client.tsx` template added (using `hydrateRoot`)
- `rolldown.config.ts` changed to array (dual build: server.js + client.js)
- `server.tsx` serializes `window.__FORTE_PROPS__` + adds `<script>` tag
- `escapeJsonForScript()` function added for XSS prevention
- `forte dev`: copies `fe/dist/client.js` → `fe/public/client.js` after build
- `forte build`: copies `dist/public/client.js`
- E2E tests passing

## Next Steps (Phase 2)

---

### 11. Production Asset Hashing ✅

Filename hashing for cache invalidation implementation complete.

- Using `std::hash::DefaultHasher` (no external dependencies)
- `client.js` → `client.{hash}.js` transformation
- Auto path substitution in `server.js`
- `dist/public/manifest.json` generation
- E2E tests passing

---

### 12. Client HMR (Hot Module Replacement) ✅

Auto browser refresh (LiveReload) on file change during development complete.

**Phase 1: LiveReload Implementation Complete**

- WebSocket server using `tokio-tungstenite`
- WebSocket upgrade at `/__hmr` endpoint
- Multi-client support via `HmrBroadcaster` (broadcast channel)
- File change → build complete → send "reload" message via WebSocket
- Client calls `location.reload()`
- Auto reconnection (exponential backoff)

**Changed files:**
- `Cargo.toml`: Added tokio-tungstenite, futures-util
- `src/server/hmr.rs`: HmrBroadcaster implementation
- `src/server/mod.rs`: WebSocket upgrade handler
- `src/cli/dev.rs`: Send reload signal on build complete
- `src/cli/init.rs`: Added HMR client code to client.tsx

**Phase 2: Vite Integration (In Progress)**

Switched to Vite instead of Rolldown + OXC for true React Fast Refresh support.

**Architecture:**
```
[forte dev]
  → codegen (forte-rs-to-ts, routes.generated.ts)
  → cargo build (WASM)
  → npx vite build --ssr (server.js for SSR)
  → npx vite (dev server on random port)
  → Forte server start (port 3000)
      ├── SSR: WASM execution → props → server.tsx execution → HTML
      ├── Proxy: /@vite/*, /src/*, /@react-refresh → Vite
      └── Static: /public/* → static files
  → Watch (backend only) → WASM rebuild → WebSocket reload
  → Frontend changes → Vite handles HMR automatically
```

**Completed work:**
- `init.rs`: Updated vite.config.ts, client.tsx, server.tsx templates
- `init.rs`: Removed scripts from package.json (Vite is an internal implementation detail)
- `server/mod.rs`: Added `dev_mode`, `fe_dir` settings
- `server/mod.rs`: Vite process startup (`npx vite`)
- `server/mod.rs`: Vite ready wait (`wait_for_vite_ready`)
- `server/mod.rs`: Vite proxy logic (`should_proxy_to_vite`, `proxy_to_vite`)
- `dev.rs`: SSR server.js build (`npx vite build --ssr --mode development`)
- `dev.rs`: Removed frontend watch from watch loop (Vite handles HMR)
- `Cargo.toml`: Added reqwest, http dependencies

**Remaining work:**
- [ ] E2E test: Verify SSR works
- [ ] E2E test: Verify React Fast Refresh works
- [ ] Vite process cleanup (terminate Vite when Forte exits)
- [ ] Update `forte build` (use Vite for production builds)

**Current issues:**
- Port conflicts during testing (leftover Vite processes)
- Vite process lifecycle management needed

**Resolution direction:**
1. Store Vite child process handle in ServerHandle
2. Terminate Vite process when Forte exits (Ctrl+C, etc.)
3. Or run Vite in the same process group for automatic cleanup

---

## Implementation Priority

| Order | Feature | Status | Difficulty |
|-------|---------|--------|------------|
| 1 | Hydration support | ✅ Complete | Medium |
| 2 | Asset hashing | ✅ Complete | Low |
| 3 | Client HMR (LiveReload) | ✅ Complete | Medium |
| 4 | Vite integration (React Fast Refresh) | 🚧 In Progress | Medium |

## Technical Decisions

### Dependency Paths
- `forte-rs-to-ts`: Relative path based on `env!("CARGO_MANIFEST_DIR")`
- `forte-json`: Path dependency (development environment)
- Removed `RUSTUP_TOOLCHAIN` env var to use rust-toolchain.toml

### Frontend Bundling
- Using Vite (dev: HMR, build: production bundle)
- `globalThis.handler` pattern for exposing global handler
- SSR build: `npx vite build --ssr src/server.tsx`
- Client build: `npx vite build` (production) / Vite dev server (development)

### Backend Package
- Package name: `backend` (fixed)
- WASM file: `rs/target/wasm32-wasip2/release/backend.wasm`
