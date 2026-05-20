# `forte init` Specification

Status: **DRAFT v2 — Decisions resolved; awaiting implementation**.

This document is the single source of truth for what `forte init <name>` must produce. The current implementation (`forte/cli/src/cli/init.rs`) diverges from this spec in known ways tracked in §8 *Decisions*; closing those gaps is the implementation work that follows this spec.

Note on related documents:
- `forte/cli/PLAN.md` and `forte/cli/PROGRESS.md` are **historical design notes** from the original CLI bring-up. They contain stale decisions that the running apps no longer follow (e.g. `rs/Cargo.toml [package].name = "backend"` as a fixed value; rolldown; the `[project].name` Forte.toml schema). Where they disagree with this spec, **this spec wins**. PLAN.md / PROGRESS.md are not normative for this work.

Ground truth for the spec is the union of:
- Hard requirements deduced by reading every CLI subcommand and `forte-codegen` (see §6 *Why each file is required*).
- The two running applications:
  - `~/fn0/fn0/control/` (Forte project for the fn0 control plane).
  - `~/amgi/web/` (Forte project for mottomite).

Where the two apps disagree, this spec takes the most minimal layout that still satisfies the CLI's hard requirements. Project-specific extensions (Tailwind, custom Vite config, env codegen, etc.) are explicitly out of init's scope.

---

## 1. Goals and non-goals

### Goals
- `forte init <name>` produces a directory `<name>/` that can immediately run, in order, with no manual edits:
  1. `cd <name> && forte dev` — boots a working dev server serving a hello-world page.
  2. `forte build` — produces `<name>/dist/{backend.wasm,server.js}` with zero errors.
  3. `forte deploy [--name X]` — succeeds against a control plane the user is logged in to.
  4. `forte add page foo` and `forte add action bar` — add a working page / action that compiles.
- All hardcoded paths and filenames in `forte-codegen`, `forte-rs-to-ts`, `forte dev`, `forte build`, and `forte deploy` resolve correctly without manual fixup.
- The output is buildable without a parent `Cargo.toml` workspace (the project is a standalone, self-contained directory).

### Non-goals
- Init does NOT install a Rust toolchain, a Node.js runtime, or the bundled `forte-rs-to-ts` binary. Those are responsibilities of the developer's environment or of other CLI subcommands.
- Init does NOT pre-create any file that the CLI itself generates on the first `forte dev` / `forte build` (those files are listed in §5).
- Init does NOT pick a styling solution, a state management library, a router, a meta-framework, an ORM, or any opinion beyond what the CLI's codegen makes mandatory.
- Init does NOT bake a `Forte.toml` `project_id` — that is filled in by `forte deploy` on first deploy.

---

## 2. CLI invocation contract

```
forte init <name>
```

- `<name>` is a non-empty string usable as a directory name AND as a Cargo package name (kebab-case allowed, will be used verbatim as the `name = "..."` value of `rs/Cargo.toml`'s `[package]`).
- If `<name>` already exists as a directory, init aborts with a clear error and does not write anything.
- On success, init prints (verbatim, last lines):
  ```
  Created project '<name>'

  Next steps:
    cd <name>
    forte dev
  ```

---

## 3. Project layout

The complete tree that init writes (◆ = file written by init, generated directories like `target/`, `dist/`, `.forte/`, `node_modules/`, `fe/dist/` are NOT created — Rust / Vite / Forte CLI will create them on first build):

```
<name>/
├── .gitignore                ◆
├── Forte.toml                ◆  (empty file; populated by forte deploy)
├── rs/
│   ├── .cargo/
│   │   └── config.toml       ◆
│   ├── .gitignore            ◆
│   ├── Cargo.toml            ◆
│   ├── build.rs              ◆
│   └── src/
│       ├── lib.rs            ◆
│       └── pages/
│           └── index/
│               └── mod.rs    ◆
└── fe/
    ├── .gitignore            ◆
    ├── package.json          ◆
    ├── tsconfig.json         ◆
    ├── public/
    │   └── robots.txt        ◆
    └── src/
        ├── app.tsx           ◆
        └── pages/
            └── index/
                └── page.tsx  ◆
```

Hard requirements driving this shape:
- The project root MUST NOT contain a `Cargo.toml`. The Rust crate at `rs/` is a standalone crate, not a workspace member. Both running apps confirm this.
- The `rs/` and `fe/` directory names are hardcoded across `forte-codegen` (`generate_routes.rs:31` derives `<manifest_dir>/../fe/src/paths.generated.ts`), `forte-rs-to-ts` (`main.rs:56-58` writes to `../fe/src/{pages,hooks/.generated,actions/.generated}`), and `forte build`/`forte dev`. They are NOT customizable.
- `fe/src/app.tsx` MUST exist — its presence is the literal trigger for `is_new_mode()` (`forte/cli/src/cli/fe_runtime.rs:5-7`), which is the codepath all current CLI logic targets. Without it, the CLI falls back to a legacy mode that init does not support.

---

## 4. File-by-file content

All file contents below are normative. They are derived from the requirements of the consuming CLI code, not copied from either running app verbatim.

### 4.1 `<name>/.gitignore`

```
/target
/dist
/.forte
```

Rationale: `/target` covers the case where the user runs `cargo` at the root; `/dist` is written by `forte build`; `/.forte` is written by `forte-rs-to-ts` (`<project>/.forte/rs-to-ts-target/`).

### 4.2 `<name>/Forte.toml`

```
```

(Zero-byte file.) `forte deploy` deserializes this as `ForteConfig { project_id: Option<String> }`; an empty file is valid and equivalent to `project_id = None`. On first deploy, the CLI writes `project_id = "..."` into this file.

### 4.3 `<name>/rs/.gitignore`

```
/target
```

### 4.4 `<name>/rs/.cargo/config.toml`

```toml
[build]
target = "wasm32-wasip2"
```

Rationale: `forte build` and `forte dev` both pass `--target wasm32-wasip2` explicitly when invoking cargo (see invariant table §6), so this file is not strictly required by the CLI itself. It IS required for the common case of a user running `cargo check` / `cargo clippy` / `cargo build` directly inside `rs/` without thinking about targets. Both running apps either ship this file (fn0/control) or fail those direct-cargo commands (amgi/web has no `.cargo/config.toml` and direct `cargo build` in `rs/` would attempt the host triple, which `forte-sdk` does not support). We pick the explicit version.

### 4.5 `<name>/rs/Cargo.toml`

Template (with `<name>` substituted and dependency sources resolved per §8.2):

```toml
[package]
name = "<name>"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
anyhow = "1"
cookie = "0.18"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
http = "1"
tracing = "0.1"
forte-json = "<version>"
forte-sdk = "<version>"
doc-db = { package = "fn0-doc-db", version = "<version>" }

[build-dependencies]
forte-codegen = "<version>"
```

The `rs/Cargo.toml` ALSO carries an empty `[workspace]` table at the top, ahead of `[package]`. This makes the crate its own single-member workspace and prevents cargo from walking up the filesystem to discover a parent workspace (the original v1 draft missed this — adding it is mandatory; without it, projects created inside an existing workspace get pulled into the parent and codegen's `--target` assumptions break).

When the CLI is built with the `--dev` flag (or equivalent build-time switch — see §8.2), the four `forte-*`/`doc-db` lines instead use `path = "<absolute path>"` resolved from the CLI's own `CARGO_MANIFEST_DIR`.

Hard requirements:
- The crate's `[package].name` is read by `find_wasm_binary` in both `dev.rs` and `build.rs`. The wasm artifact path is derived as `target/wasm32-wasip2/release/<name with - replaced by _>.wasm`. The name is **user-supplied via the CLI argument**, not the fixed string `"backend"` (this overrides the stale convention in `PROGRESS.md`).
- `crate-type = ["cdylib"]` is mandatory; `forte-codegen` generates a `cdylib` entry point against the WASI HTTP world.
- `edition = "2024"` matches what `forte-codegen` formats output for (`rustfmt --edition=2024`).
- `doc-db` MUST be declared with the `package = "fn0-doc-db"` rename so that user code reads `use doc_db::...;` while the published crate name is `fn0-doc-db` (`doc-db/Cargo.toml:2`).

`tracing` IS included even though the seed index handler does not reference it: `forte-codegen` emits `#[forte_sdk::tracing::instrument(...)]` attributes on the generated dispatcher functions, and the expansion calls `<future>.instrument(span)` — which needs the `tracing::Instrument` trait visible at the call site. Without `tracing` in the user crate, the first `forte build` fails with `E0433: failed to resolve: use crate::route_generated::tracing::Instrument`. (The original v1 draft excluded `tracing`; e2e proved that to be a load-bearing dep.)

NOT included in the template:
- `serde_with`, `chrono`, `base64`, etc. — application-specific.

### 4.6 `<name>/rs/build.rs`

```rust
fn main() {
    forte_codegen::generate_routes();
}
```

Hard requirement: `forte-codegen::generate_routes()` writes `rs/src/route_generated.rs` and rewrites the FORTE-MANAGED block in `rs/src/lib.rs`. Without this call, the project will not compile after init.

NOT included: `forte_codegen::generate_env()`. It is opt-in (requires a `<name>/.env` file). amgi/web uses it; fn0/control does not. Users who want `.env`-derived constants add the call themselves.

### 4.7 `<name>/rs/src/lib.rs`

```rust
// === FORTE-MANAGED START ===
// Auto-managed by `forte build`. Do not edit between the START/END markers.
mod route_generated;
// === FORTE-MANAGED END ===
```

Hard requirements:
- The two literal marker comments are matched by string in `forte/codegen/src/generate_routes.rs:101-107`. **`generate_routes()` PANICS if either marker is missing.**
- The body between markers is overwritten on every build by codegen. Init writes a minimal seed (`mod route_generated;`) so that the marker block is well-formed for codegen to find on the very first build.

**First-compile note (decision §8.1 option A):** Immediately after `forte init`, the file `rs/src/route_generated.rs` does NOT yet exist. The seed `mod route_generated;` line therefore points at a non-existent file, and a *standalone* `cargo check` / `cargo clippy` in `rs/` will fail. The first action after init MUST be `forte dev` or `forte build`; their build-script step runs `forte-codegen::generate_routes()` which writes `route_generated.rs` **before** rustc compiles the crate, so the first build succeeds. This is the same gap both running apps have; the CLI's "Next steps" message tells the user the correct first action.

The CLI's two managed comments inside the marker block are tolerated by the comment-style rules: they are non-obvious, encode a real invariant (machine-managed boundary), and would confuse a reader who tries to edit between them.

### 4.8 `<name>/rs/src/pages/index/mod.rs`

```rust
use anyhow::Result;
use forte_sdk::ForteRequest;
use serde::Serialize;

#[derive(Serialize)]
pub enum Props {
    Ok { message: String },
}

pub async fn handler(_req: ForteRequest<'_>) -> Result<Props> {
    Ok(Props::Ok {
        message: "Hello from Forte!".to_string(),
    })
}
```

Hard requirements:
- Page detection in `forte/codegen/src/generate_routes/discover.rs:266-298` matches on `pub async fn handler(...) -> Result<...>` returning a type that references `Props` or `Redirect`.
- The handler signature MUST match what the generated dispatcher in `forte/codegen/src/generate_routes/codegen.rs` actually calls. The seed handler uses `ForteRequest<'_>` (matches dispatcher). The unrelated `forte add page` template in `add.rs:76-124` currently uses `(headers, jar)` — that template is stale and is tracked as a separate fix in §8.7.
- `Props` MUST be `Serialize` and the variant `Ok { message: String }` MUST exist for `forte-rs-to-ts` to emit a usable `Props` TypeScript type that the seed `page.tsx` consumes.

### 4.9 `<name>/fe/.gitignore`

```
/node_modules
/dist
/.forte
```

Rationale: `/.forte` is the runtime directory written by `fe_runtime::ensure()` and codegen.

### 4.10 `<name>/fe/package.json`

```json
{
  "name": "<name>-frontend",
  "private": true,
  "type": "module",
  "dependencies": {
    "react": "^19.2",
    "react-dom": "^19.2",
    "zod": "^4"
  },
  "devDependencies": {
    "@types/react": "^19.2",
    "@types/react-dom": "^19.2",
    "@vitejs/plugin-react": "^6",
    "typescript": "^5.9",
    "vite": "^8"
  }
}
```

Hard requirements:
- `react`, `react-dom`: the bundled `fe/.forte/{server.tsx,client.tsx}` imports them.
- `zod`: the bundled `fe/.forte/forte-react.ts` imports it.
- `@vitejs/plugin-react`, `vite`: required by the bundled `fe/.forte/vite.config.ts`.
- `typescript`, `@types/react`, `@types/react-dom`: required by `npx vite build` and `tsc` (Vite invokes TS via the plugin).

Init MUST write `package.json` first, then run `npm install` **once** in `<name>/fe/`. A single invocation is sufficient because `package.json` already lists every package; the current `init.rs:73-110` runs two separate `npm install` commands without `package.json` listing them up front, and that pattern goes away.

Version pins:
- fn0/control: `typescript ^6.0.3`. amgi/web: `typescript ^5.9.3`. We pin to `^5.9` for stability (TS 6.0 is recent and breaks some downstream tooling); revisit in a follow-up.
- React 19.2.x is consistent across both apps and is the lowest version that ships the new SSR API used by `fe_runtime/server.tsx`.

### 4.11 `<name>/fe/tsconfig.json`

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["src", ".forte"]
}
```

Differences from current `init.rs`:
- `"noEmit": true` — Vite handles emit, `tsc` is for type-checking only. amgi/web ships this; fn0/control uses `"outDir": "dist"` which conflicts with Vite's output directory.
- `"include": ["src", ".forte"]` — the runtime files in `fe/.forte/` are TS and need type checking once generated. The `.forte/` directory is absent right after init; this is harmless (tsc treats missing globs as empty).

### 4.12 `<name>/fe/public/robots.txt`

```
User-agent: *
Allow: /
```

Minimal stub. `forte build` does not require this file but `dev.rs:324` passes `<name>/fe/public/` to the dev server as `public_dir`, and Vite errors if the directory is missing.

### 4.13 `<name>/fe/src/app.tsx`

```tsx
export function Head() {
    return (
        <>
            <meta charSet="utf-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            <title>Forte App</title>
        </>
    );
}
```

Hard requirements:
- The presence of this file is what `is_new_mode()` checks (`fe_runtime.rs:5-7`).
- The export `Head` is imported by the bundled `fe/.forte/server.tsx:11` (`import { Head } from "../src/app";`). Missing the export breaks SSR.

### 4.14 `<name>/fe/src/pages/index/page.tsx`

```tsx
import type { Props } from "./.props";

export default function IndexPage(props: Props) {
    if (props.t !== "Ok") {
        return <div>Error loading page</div>;
    }

    return (
        <div>
            <h1>Welcome to Forte</h1>
            <p>{props.message}</p>
        </div>
    );
}
```

Hard requirements:
- The default export is consumed by the generated `fe/.forte/routes.generated.ts`.
- `./.props` is the file emitted by `forte-rs-to-ts` next to this `page.tsx`. The `Props` TypeScript type is a Zod-derived **flat** discriminated union: `{ t: "Ok", message: string }` — NOT `{ t, v }`. Variant fields are spread onto the top-level object rather than nested under `v`. fn0/control's index page.tsx confirms this shape. (The v1 spec draft had `{ t, v }` which is incorrect.)

---

## 5. Files init MUST NOT create (the CLI generates them)

These files appear in fresh projects after the first `forte dev` or `forte build`, NOT in the init output:

- `<name>/rs/src/route_generated.rs` — written by `forte-codegen::generate_routes()`.
- `<name>/rs/src/env_generated.rs` — written by `forte-codegen::generate_env()` if used.
- `<name>/rs/src/{actions,admin,queue_task}/mod.rs` — written by codegen when corresponding directories have at least one item.
- `<name>/fe/src/paths.generated.ts` — written by codegen.
- `<name>/fe/src/pages/**/.props.ts` and `.input.ts` etc. — written by `forte-rs-to-ts`.
- `<name>/fe/src/hooks/.generated/*`, `<name>/fe/src/actions/.generated/*` — written by `forte-rs-to-ts`.
- `<name>/fe/.forte/{server.tsx,client.tsx,vite.config.ts,forte-react.ts,routes.generated.ts}` — written by `fe_runtime::ensure()` and codegen.
- `<name>/fe/dist/`, `<name>/dist/`, `<name>/rs/Cargo.lock`, `<name>/rs/target/`, `<name>/.forte/rs-to-ts-target/` — built artifacts.

The current `init.rs` does not create any of these — that part is correct. This section is normative for future changes.

---

## 6. Why each file is required (cross-cutting invariants)

A future contributor changing the CLI MUST preserve these invariants or update this spec:

| Invariant | Enforced by |
| --- | --- |
| Root has no `Cargo.toml` | `forte-rs-to-ts/src/main.rs:75` sets `current_dir = <project>/rs`; both running apps. |
| `rs/` is the Rust crate root | `forte-codegen/src/generate_routes.rs:16` (`CARGO_MANIFEST_DIR`), `forte-rs-to-ts/src/main.rs:66`. |
| `fe/` is sibling of `rs/` | `forte-codegen/src/generate_routes.rs:31` (`../fe/src/paths.generated.ts`), `forte-rs-to-ts/src/main.rs:56-58`. |
| `fe/src/app.tsx` exists and exports `Head` | `forte/cli/src/cli/fe_runtime.rs:5-7`; bundled `fe_runtime/server.tsx:11`. |
| `rs/src/lib.rs` has both `// === FORTE-MANAGED START ===` and `// === FORTE-MANAGED END ===` markers | `forte-codegen/src/generate_routes.rs:101-107` panics if missing. |
| `rs/Cargo.toml` has `[lib] crate-type = ["cdylib"]` and `[package].name = <name>` | `forte/cli/src/cli/build.rs` and `dev.rs` (`find_wasm_binary`). |
| `rs/Cargo.toml`'s `doc-db` dep uses `package = "fn0-doc-db"` | `doc-db/Cargo.toml:2`. |
| `Forte.toml` exists (even if empty) | `forte/cli/src/cli/build.rs:17-19` hard-fails if missing. |
| The CLI invokes cargo with `--target wasm32-wasip2` explicitly | `forte/cli/src/cli/build.rs:215-222`, `dev.rs:215-222`. The project's `rs/.cargo/config.toml` is therefore for developer ergonomics only, not for the CLI's correctness. |

---

## 7. Side effects init performs

In addition to writing the files in §4, init performs these actions in this order:

1. Create directory `<name>/` and all subdirectories listed in §3.
2. Write all files listed in §4.
3. Run `npm install` **once** in `<name>/fe/` (creates `<name>/fe/node_modules/` and `<name>/fe/package-lock.json`). This is the only network call init makes. If `npm install` fails, init aborts with a non-zero exit code; partially-written files remain on disk (init does NOT roll back).
4. Print the "Next steps" message (§2).

Init does NOT run `cargo` (no first-build trigger), does NOT download `forte-rs-to-ts` (deferred until `forte dev`/`build`), does NOT initialize git.

---

## 8. Decisions

All previously-open items are resolved. Each decision below is binding on the implementation.

### 8.1 First-compile bootstrap of `rs/src/lib.rs` ✅ Decided

**Decision: option A — Accept the gap.** Init writes `mod route_generated;` between the FORTE-MANAGED markers even though `route_generated.rs` does not yet exist. The first action after `forte init` is `forte dev` or `forte build`; the build-script step writes `route_generated.rs` before rustc compiles the crate.

Implication for §9 acceptance: `cargo clippy` in `rs/` is verified AFTER the first `forte build` has run, not on a pristine init output.

### 8.2 Path deps vs. published-crate deps in `rs/Cargo.toml` ✅ Decided

**Decision: option B — emit version deps by default; offer `--dev` for path deps.**

- Default `forte init <name>` writes `forte-json = "<v>"`, `forte-sdk = "<v>"`, `doc-db = { package = "fn0-doc-db", version = "<v>" }`, `forte-codegen = "<v>"` — i.e. version deps from crates.io. This is the form that works when the user installs `forte` via `cargo install` or homebrew.
- `forte init --dev <name>` (new flag) writes `path = "<absolute path>"` deps resolved from the CLI binary's `CARGO_MANIFEST_DIR`, matching today's behavior. This is for in-repo development inside the fn0 monorepo only.
- The exact pinned versions baked into the default template are read from the workspace `Cargo.toml` at CLI-build time (single source of truth). Today's pinned versions: `forte-json = 0.1.1`, `forte-sdk = 0.3.5`, `forte-codegen = 0.1.3`, `fn0-doc-db = 0.4.2`.

### 8.3 `forte-rs-to-ts` version pin unification ✅ Decided

**Decision: unify on `0.1.8` and publish a matching binary release.**

- `forte/cli/src/cli/build.rs:38` already pins `0.1.8`.
- `forte/cli/src/cli/dev.rs:40` is updated from `0.1.7` to `0.1.8` (this commit).
- `forte/rs-to-ts/Cargo.toml:3` already declares `version = "0.1.8"`.
- A new tag `forte-rs-to-ts-v0.1.8` is pushed; the `release-forte-rs-to-ts.yml` GitHub Actions workflow then builds and publishes the cross-platform archives that both CLI subcommands resolve via `fn0_release_url`.

Rationale: source version, CLI pins, and the GitHub release MUST all match, because `tools::fn0_release_url` synthesizes the download URL from the version constant. The current 0.1.7/0.1.8 split is a latent bug — uncached environments would 404 on `forte build`.

### 8.4 `wasm-component-ld` toolchain note ✅ Decided

**Decision: document the requirement in the fn0 README; do NOT add a runtime check to init.**

The Forte CLI invokes `cargo build --target wasm32-wasip2` and relies on cargo's native component linking. Stable Rust ≥ 1.84 ships a `wasm-component-ld` that decodes `wit-bindgen 0.50.0` output; older toolchains do not. This is an environment requirement, not an init concern.

### 8.5 Should init initialize git? ✅ Decided

**Decision: no.** Users who want git run `git init` themselves. Init stays minimal.

### 8.6 Should init create `<name>/env.yaml` and `<name>/.env`? ✅ Decided

**Decision: no.** Empty optional files are noise. Document them in `forte deploy`'s help output and in the README.

### 8.7 `forte add` templates are stale ✅ Decided (fixed in the init rewrite PR)

Three problems in `forte/cli/src/cli/add.rs`, all fixed together:

1. **Page backend template**: was `pub async fn handler(_headers: HeaderMap, _jar: CookieJar) -> Result<Props>`. The dispatcher calls `handler(req: ForteRequest)`. Fixed to use `ForteRequest<'_>` (and `(req, _path_params: PathParams)` for dynamic-segment pages, with the param struct renamed from `Params` to `PathParams` to match codegen's `discover.rs`).

2. **Action backend template**: was `pub async fn action(input: <Name>Input) -> Result<<Name>Output>`. `forte-codegen::discover_actions` requires the names `Input`, `Output`, and `handler`. Fixed to `pub async fn handler(req: ForteRequest<'_, Input>) -> Output` with the canonical type names.

3. **Action frontend wrapper removed entirely**: `forte-rs-to-ts` already emits a fully-typed Zod + `callAction()` wrapper at `fe/src/actions/.generated/<name>.ts` (both running apps consume actions exclusively via that path — `import { submit } from "../../actions/.generated/submit"`). The hand-rolled wrapper that `forte add action` used to write to `fe/src/actions/<name>.ts` was redundant, used stale type names (`<Name>Input`/`<Name>Output`), and imported from a path (`./{action_path}.types`) that does not exist. `forte add action` now writes only the backend `.rs` file and prints the import path the user should use.

4. **Action frontend wrapper URL**: the now-removed wrapper had `/_action/<path>` instead of `/__forte_action/<path>`. Moot post-removal, but worth flagging for anyone restoring the wrapper.

### 8.9 Default-mode (version-deps) build needed `fn0-doc-db 0.4.3` publish ✅ Resolved

The published `fn0-doc-db 0.4.2` on crates.io pinned `forte-sdk = "=0.3.4"` while the workspace had moved to `forte-sdk 0.3.5`. Default-mode `forte init` produced a project whose first `forte build` failed dependency resolution:

```
error: failed to select a version for `forte-sdk`.
    ... required by package `fn0-doc-db v0.4.2`
versions that meet the requirements `=0.3.4` are: 0.3.4
previously selected package `forte-sdk v0.3.5`
```

Resolved by bumping `doc-db/Cargo.toml` 0.4.2 → 0.4.3 and `cargo publish -p fn0-doc-db`. The 0.4.3 release on crates.io now carries `forte-sdk = "=0.3.5"`. E2E confirmed: `forte init demo` (default mode) followed by `forte build` now resolves cleanly from crates.io.

### 8.10 `forte` symlink targeted the wrong target directory ✅ Resolved

`forte/cli/build.rs` previously synthesized the symlink target as `<manifest_dir>/target/<profile>/forte`, which only existed when forte/cli was built standalone. Workspace builds (the default) put the binary at `<workspace_root>/target/<profile>/forte`, so the symlink pointed at a stale path. Fixed: `build.rs` now walks up from `CARGO_MANIFEST_DIR` to find the nearest `Cargo.toml` containing `[workspace]` and uses that root's `target/` (honors `CARGO_TARGET_DIR` if set).

### 8.8 Relationship to `PLAN.md` / `PROGRESS.md` ✅ Decided

Those documents are **historical** and contain stale decisions (`name = "backend"` fixed; rolldown; `[project].name` Forte.toml schema). When this spec is accepted, add a header line to both files: "Historical design notes. For the current `forte init` specification, see `INIT_SPEC.md`." Do NOT delete them; they carry useful context for the rest of the CLI surface (HMR, hot swap, etc.). Do NOT propagate their stale init claims into new code.

---

## 9. Acceptance criteria for the implementation PR

The `init.rs` rewrite is complete when, in a clean directory with no prior `forte` cache:

1. `forte init demo` writes exactly the tree in §3 plus `fe/node_modules/` and `fe/package-lock.json`, and runs `npm install` exactly **once**.
2. `forte dev` boots without error and serves `http://localhost:<port>/` showing "Welcome to Forte" + "Hello from Forte!".
3. `forte build` exits 0 and `ls dist/` shows `backend.wasm` and `server.js`.
4. `forte deploy --name demo-test` against a logged-in control plane succeeds end-to-end and writes `project_id` into `Forte.toml`.
5. `forte add page about && forte add action submit` writes the expected files and `forte build` still passes (this implicitly validates the §8.7 `add.rs` template fix).
6. After the first `forte build`, running `cargo clippy --release --target wasm32-wasip2` inside `rs/` exits 0 with zero warnings. (`cargo clippy` invokes `build.rs`, which writes `route_generated.rs` before rustc compiles the crate, so clippy passes even on a freshly-initted project — but only after build artifacts are in place; on the pristine init output before any build, clippy is expected to fail per §8.1.)
7. `forte init --dev demo-dev` writes path-dep variants of the four `forte-*`/`doc-db` lines in `rs/Cargo.toml` and is buildable inside the fn0 monorepo without further edits.

Each criterion must be reproduced once and recorded in the PR description.

## 10. Verification log

All seven §9 criteria reproduced and passing as of 2026-05-20:

1. ✅ `forte init --dev demo-dev` produced exactly the §3 tree plus `fe/node_modules/` + `fe/package-lock.json`; `npm install` ran once.
2. ✅ `forte dev` served `http://localhost:3000/` and `/about` with HTTP 200 and the expected hello-world content.
3. ✅ `forte build` exited 0; `dist/` contained `backend.wasm` (1.6 MB) + `server.js`.
4. ✅ `forte deploy --name demo-test` succeeded end-to-end, wrote `project_id = "7d4eez0e"` into `Forte.toml`, uploaded 4 static assets + the bundle tar; the deployed site is live at `https://7d4eez0e.fn0.dev/`.
5. ✅ `forte add page about` + `forte add action submit` + `forte build` passed; `POST /__forte_action/submit` returned `{"t":"Ok","result":"Received: hi"}`.
6. ✅ `cargo clippy --release --target wasm32-wasip2` in `rs/` exited 0 with zero warnings (after the first build).
7. ✅ `forte init --dev demo-dev` emitted path-deps and built inside the monorepo; default-mode `forte init demo` emitted version-deps and built cleanly from crates.io after `fn0-doc-db 0.4.3` was published (§8.9).
