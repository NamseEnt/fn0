# HMR + React Fast Refresh Implementation Plan for Ski Dev Environment

## Goal

Use Ski in the dev environment without Vite, while providing the same HMR (Hot Module Replacement) and React Fast Refresh capabilities that Vite offers.

---

## 1. Background

### 1.1 What is HMR (Hot Module Replacement)?

A technique that replaces only changed modules without a full page reload when files change.

**Flow:**
1. File system watcher detects file changes
2. Server finds the changed file in the module graph
3. Determines affected modules
4. Instructs the client to update via WebSocket
5. Client fetches new module → replaces existing module

**Key Concepts:**
- **HMR Boundary**: A module that "accepts" an update. The update scope is determined based on this boundary
- **Module Graph**: A graph tracking import relationships between modules

### 1.2 What is React Fast Refresh?

React's official HMR solution. Updates code while **preserving component state**.

**How it works:**
1. Babel/SWC plugin transforms code to inject component registration code
2. `react-refresh/runtime` manages the component registry
3. On HMR, only changed components re-render (state preserved)

**Transform example:**
```jsx
// Original code
function Counter() {
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount(c => c + 1)}>{count}</button>;
}

// Transformed code
var _s = $RefreshSig$();
function Counter() {
  _s();
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount(c => c + 1)}>{count}</button>;
}
_s(Counter, "useState{[count, setCount](0)}");
$RefreshReg$(Counter, "Counter");
```

**Limitations:**
- Class components are not supported (function components + hooks only)
- `// @refresh reset` comment forces remount

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           FORTE DEV SERVER (Rust)                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────────────────────┐ │
│  │   Watcher    │──▶│  HMR Engine  │──▶│     WebSocket Server         │ │
│  │  (notify)    │   │              │   │  (extend existing /__hmr)    │ │
│  └──────────────┘   └──────┬───────┘   └──────────────────────────────┘ │
│                            │                                            │
│                            ▼                                            │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                     Module Graph                                  │   │
│  │  - Track import/export relationships between modules              │   │
│  │  - Manage HMR boundary information                                │   │
│  │  - Hash of each module (for change detection)                     │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                     Transform Pipeline                            │   │
│  │                                                                   │   │
│  │  On .tsx/.ts file request:                                        │   │
│  │  1. TypeScript/JSX transpile (esbuild/swc)                        │   │
│  │  2. React Fast Refresh transform (swc plugin)                     │   │
│  │  3. HMR client code injection                                     │   │
│  │  4. Import path rewrite (bare → actual path)                      │   │
│  │                                                                   │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                     Dependency Pre-bundler                        │   │
│  │  - Scan node_modules dependencies                                 │   │
│  │  - Generate ESM bundles with esbuild                              │   │
│  │  - Cache in /.forte/deps/                                         │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                              BROWSER                                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                     HMR Client Runtime                            │   │
│  │  - WebSocket connection management                                │   │
│  │  - Provide import.meta.hot API                                    │   │
│  │  - Module replacement logic                                       │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                   React Refresh Runtime                           │   │
│  │  - Component registry ($RefreshReg$)                              │   │
│  │  - Hook signature tracking ($RefreshSig$)                         │   │
│  │  - Call performReactRefresh() on update                           │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Components to Implement

### 3.1 Server Side (Rust - forte cli)

#### 3.1.1 Transform Pipeline

**Location:** `forte/cli/src/transform/`

```rust
// transform/mod.rs
pub struct TransformPipeline {
    swc_compiler: SwcCompiler,
    module_graph: Arc<RwLock<ModuleGraph>>,
    dep_cache: Arc<RwLock<DependencyCache>>,
}

impl TransformPipeline {
    /// Transform .tsx/.ts files to browser-ready JS
    pub async fn transform(&self, file_path: &Path, is_ssr: bool) -> Result<TransformResult> {
        // 1. Read file
        // 2. Transpile with SWC (including React Refresh plugin)
        // 3. Import path rewrite
        // 4. HMR code injection (if not SSR)
        // 5. Update module graph
    }
}

pub struct TransformResult {
    pub code: String,
    pub map: Option<String>,  // sourcemap
    pub deps: Vec<String>,    // imported modules
}
```

#### 3.1.2 Module Graph

**Location:** `forte/cli/src/module_graph/`

```rust
// module_graph/mod.rs
pub struct ModuleGraph {
    modules: HashMap<PathBuf, Module>,
}

pub struct Module {
    pub id: String,              // URL path (e.g., /src/components/Button.tsx)
    pub file_path: PathBuf,      // actual file path
    pub importers: HashSet<String>,  // modules that import this module
    pub imports: HashSet<String>,    // modules this module imports
    pub accepts_hmr: bool,           // whether import.meta.hot.accept() is called
    pub is_self_accepting: bool,     // whether it accepts itself
    pub hash: String,                // content hash (for change detection)
    pub transform_result: Option<TransformResult>,
}

impl ModuleGraph {
    /// Calculate affected modules on file change
    pub fn get_hmr_boundaries(&self, changed_file: &Path) -> Vec<HmrUpdate> {
        // 1. Start from the changed file
        // 2. Traverse up through importers
        // 3. Stop when finding a module with accepts_hmr=true (boundary)
        // 4. If no boundary reached, full reload needed
    }
}

pub struct HmrUpdate {
    pub boundary: String,      // HMR boundary module ID
    pub modules: Vec<String>,  // modules that need updating
    pub full_reload: bool,     // whether full reload is needed
}
```

#### 3.1.3 HMR Engine

**Location:** `forte/cli/src/hmr/`

```rust
// hmr/mod.rs
pub struct HmrEngine {
    module_graph: Arc<RwLock<ModuleGraph>>,
    broadcaster: HmrBroadcaster,
}

impl HmrEngine {
    /// Handle file change
    pub async fn handle_file_change(&self, file_path: &Path) -> Result<()> {
        // 1. Find HMR boundaries in module graph
        let updates = self.module_graph.read().get_hmr_boundaries(file_path);

        // 2. Re-transform changed modules
        for update in &updates {
            self.retransform_modules(&update.modules).await?;
        }

        // 3. Send HMR payload
        let payload = HmrPayload::Update {
            updates,
            timestamp: SystemTime::now(),
        };
        self.broadcaster.send(payload).await;

        Ok(())
    }
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum HmrPayload {
    #[serde(rename = "update")]
    Update {
        updates: Vec<HmrUpdate>,
        timestamp: u64,
    },
    #[serde(rename = "full-reload")]
    FullReload {
        path: Option<String>,
    },
    #[serde(rename = "prune")]
    Prune {
        paths: Vec<String>,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
        stack: Option<String>,
    },
}
```

#### 3.1.4 Dependency Pre-bundler

**Location:** `forte/cli/src/deps/`

```rust
// deps/mod.rs
pub struct DependencyPrebundler {
    cache_dir: PathBuf,  // .forte/deps/
}

impl DependencyPrebundler {
    /// Bundle node_modules dependencies as ESM
    pub async fn prebundle(&self, project_root: &Path) -> Result<DependencyMap> {
        // 1. Read dependencies from package.json
        // 2. Scan actually used dependencies (import analysis)
        // 3. Bundle each dependency with esbuild
        //    - CommonJS → ESM conversion
        //    - Bundle internal imports (multiple files within react)
        // 4. Generate .forte/deps/react.js, .forte/deps/react-dom.js, etc.
        // 5. Return import map
    }
}

pub struct DependencyMap {
    pub entries: HashMap<String, String>,  // "react" → "/.forte/deps/react-xxx.js"
}
```

#### 3.1.5 SWC Integration

**Location:** `forte/cli/src/transform/swc.rs`

```rust
// transform/swc.rs
use swc_core::{
    ecma::{
        parser::{Syntax, TsSyntax},
        transforms::react::{react, refresh},
    },
};

pub struct SwcCompiler {
    cm: Arc<SourceMap>,
}

impl SwcCompiler {
    pub fn transform_with_refresh(&self, code: &str, filename: &str) -> Result<TransformOutput> {
        // SWC configuration
        let config = swc_core::ecma::transforms::react::Options {
            runtime: Some(Runtime::Automatic),
            development: Some(true),
            refresh: Some(RefreshOptions {
                refresh_reg: "$RefreshReg$".into(),
                refresh_sig: "$RefreshSig$".into(),
                emit_full_signatures: true,
            }),
            ..Default::default()
        };

        // Perform transformation
        // 1. TypeScript → JavaScript
        // 2. JSX → React.createElement (or jsx-runtime)
        // 3. React Refresh code injection
    }
}
```

### 3.2 Client Side (JavaScript)

#### 3.2.1 HMR Client Runtime

**Location:** `forte/cli/assets/hmr-client.js`

```javascript
// hmr-client.js
// Injected into HTML and runs in the browser

class HMRClient {
  constructor() {
    this.socket = null;
    this.moduleMap = new Map();  // moduleId → { module, callbacks }
    this.connect();
  }

  connect() {
    this.socket = new WebSocket(`ws://${location.host}/__hmr`);

    this.socket.onmessage = (event) => {
      const payload = JSON.parse(event.data);
      this.handleMessage(payload);
    };

    this.socket.onclose = () => {
      // Reconnection logic
      setTimeout(() => this.connect(), 1000);
    };
  }

  handleMessage(payload) {
    switch (payload.type) {
      case 'update':
        this.handleUpdate(payload);
        break;
      case 'full-reload':
        location.reload();
        break;
      case 'error':
        this.handleError(payload);
        break;
    }
  }

  async handleUpdate(payload) {
    for (const update of payload.updates) {
      if (update.full_reload) {
        location.reload();
        return;
      }

      // Fetch new modules
      for (const moduleId of update.modules) {
        const newUrl = `${moduleId}?t=${payload.timestamp}`;
        const newModule = await import(newUrl);

        // Execute registered callbacks
        const entry = this.moduleMap.get(moduleId);
        if (entry && entry.callbacks.length > 0) {
          for (const cb of entry.callbacks) {
            cb(newModule);
          }
        }
      }

      // Trigger React Refresh
      if (window.$RefreshRuntime$) {
        window.$RefreshRuntime$.performReactRefresh();
      }
    }
  }

  // import.meta.hot API implementation
  createHotContext(moduleId) {
    const hot = {
      accept: (deps, callback) => {
        if (typeof deps === 'function' || deps === undefined) {
          // Self-accepting: import.meta.hot.accept()
          this.registerModule(moduleId, { selfAccept: true, callback: deps });
        } else {
          // Dep-accepting: import.meta.hot.accept(['./foo'], cb)
          this.registerModule(moduleId, { deps, callback });
        }
      },

      dispose: (callback) => {
        // Cleanup to run when module is removed
        this.registerDispose(moduleId, callback);
      },

      invalidate: () => {
        // Force propagation upward
        this.socket.send(JSON.stringify({ type: 'invalidate', path: moduleId }));
      },

      data: {},  // For passing data between HMR updates
    };

    return hot;
  }
}

// Global instance
window.__forte_hmr = new HMRClient();

// import.meta.hot polyfill
// (Used by code injected into each module)
window.__forte_createHotContext = (moduleId) => {
  return window.__forte_hmr.createHotContext(moduleId);
};
```

#### 3.2.2 React Refresh Runtime Wrapper

**Location:** `forte/cli/assets/react-refresh-runtime.js`

```javascript
// react-refresh-runtime.js
// Wraps react-refresh/runtime

import RefreshRuntime from 'react-refresh/runtime';

// Initialize React Refresh
RefreshRuntime.injectIntoGlobalHook(window);

// Register global functions (used by transformed code)
window.$RefreshReg$ = (type, id) => {
  RefreshRuntime.register(type, window.__currentModuleId + ' ' + id);
};

window.$RefreshSig$ = RefreshRuntime.createSignatureFunctionForTransform;

// Called after HMR update
window.$RefreshRuntime$ = RefreshRuntime;
```

#### 3.2.3 HMR Code Injected into Each Module

**Automatically injected into each .tsx file during transform:**

```javascript
// Injected at module start
import.meta.hot = window.__forte_createHotContext("/src/components/Button.tsx");
window.__currentModuleId = "/src/components/Button.tsx";

var $RefreshReg$ = window.$RefreshReg$;
var $RefreshSig$ = window.$RefreshSig$;

// ... original code (with React Refresh transform applied) ...

// Injected at module end
if (import.meta.hot) {
  import.meta.hot.accept();  // Register as self-accepting

  // Schedule React Refresh
  window.$RefreshRuntime$.performReactRefresh();
}
```

---

## 4. Import Path Rewrite

### 4.1 Cases to Handle

```javascript
// 1. Bare import (node_modules)
import React from 'react';
// → import React from '/.forte/deps/react-abc123.js';

// 2. Relative import
import { Button } from './Button';
// → import { Button } from '/src/components/Button.tsx?t=1234567890';

// 3. Alias import
import { utils } from '@/lib/utils';
// → import { utils } from '/src/lib/utils.ts?t=1234567890';

// 4. CSS import
import './styles.css';
// → import '/.forte/css/styles-def456.css';

// 5. Asset import
import logo from './logo.png';
// → const logo = '/.forte/assets/logo-ghi789.png';
```

### 4.2 Implementation Logic

```rust
// transform/import_rewrite.rs
pub struct ImportRewriter {
    project_root: PathBuf,
    dependency_map: DependencyMap,
    timestamp: u64,
}

impl ImportRewriter {
    pub fn rewrite(&self, source: &str, current_file: &Path) -> String {
        // 1. Parse all import statements
        // 2. Classify each import's specifier:
        //    - bare: 'react', 'react-dom/client'
        //    - relative: './foo', '../bar'
        //    - alias: '@/lib/utils'
        // 3. Transform paths according to classification
        // 4. Add timestamp query parameter (for cache invalidation)
    }
}
```

---

## 5. CSS Handling

### 5.1 Tailwind CSS

```rust
// css/tailwind.rs
pub struct TailwindProcessor {
    config_path: PathBuf,
    output_path: PathBuf,
}

impl TailwindProcessor {
    pub async fn build(&self) -> Result<()> {
        // Run tailwindcss CLI
        // npx tailwindcss -i ./src/index.css -o ./.forte/css/main.css --watch
    }

    pub async fn rebuild_on_change(&self, changed_file: &Path) -> Result<bool> {
        // Tailwind classes may have changed on .tsx/.html file changes
        // Rebuild and send HMR
    }
}
```

### 5.2 CSS HMR

```javascript
// CSS file import transform
import './Button.css';
// →
const styleId = '/.forte/css/Button-abc123.css';
const link = document.createElement('link');
link.rel = 'stylesheet';
link.href = styleId;
document.head.appendChild(link);

if (import.meta.hot) {
  import.meta.hot.accept();
  import.meta.hot.dispose(() => {
    link.remove();
  });
}
```

---

## 6. SSR Handling

### 6.1 SSR Transform (excluding Fast Refresh)

```rust
impl TransformPipeline {
    pub async fn transform_for_ssr(&self, file_path: &Path) -> Result<TransformResult> {
        // HMR/Fast Refresh not needed for SSR
        // Perform pure transpilation only
        // - TypeScript → JavaScript
        // - JSX → React.createElement
        // - Import path rewrite (keep node_modules as-is)
    }
}
```

### 6.2 SSR Module Loading

Current: `vite.ssrLoadModule()`
Replacement: Execute bundled server.js directly in `ski`

```rust
// server/ssr.rs
pub struct SsrRenderer {
    ski_runtime: SkiRuntime,
    server_bundle_path: PathBuf,
}

impl SsrRenderer {
    pub async fn render(&self, url: &str, props: Value) -> Result<String> {
        // 1. Re-bundle if server.js has changed
        // 2. Execute in ski
        // 3. Return HTML
    }

    async fn rebuild_server_bundle(&self) -> Result<()> {
        // Bundle server.tsx → server.js with esbuild
        // node_modules are external
    }
}
```

---

## 7. Phased Implementation Plan

> **Core Principle:** Finalize the target architecture from the start, and only implement incrementally.
> This way, no refactoring is needed — just "enable" features, and each phase is verifiable.

```rust
// Example: Even in Phase 1, use the final structure
pub struct TransformPipeline {
    swc_compiler: SwcCompiler,
    module_graph: Arc<RwLock<ModuleGraph>>,  // Implemented in Phase 1
    refresh_enabled: bool,                    // Set to true in Phase 2
}
```

---

### Phase 1: Transform + Module Graph + HMR

**Goal:** Replace only the changed module on file change (state is reset)

#### 1.1 SWC Integration + Transform Pipeline

```
Tasks:
├── Add swc_core crate
├── TypeScript/JSX → JavaScript transform
├── Transpile cache implementation (hash-based)
└── Test: Verify a single .tsx file runs in the browser
```

- Create `forte/cli/src/transform/mod.rs`
- Create `forte/cli/src/transform/swc.rs`
- React Refresh transform is **disabled** but structurally prepared

#### 1.2 Dependency Pre-bundling

```
Tasks:
├── Parse package.json → extract dependency list
├── Bundle each dependency as ESM with esbuild
├── Cache in .forte/deps/ directory
├── Implement import rewrite ('react' → '/.forte/deps/react-xxx.js')
└── Test: Verify import React from 'react' works
```

- Create `forte/cli/src/deps/mod.rs`
- Create `forte/cli/src/transform/import_rewrite.rs`

#### 1.3 Module Graph

```
Tasks:
├── Data structure for tracking inter-module import relationships
├── Import analysis during transform → graph update
├── HMR boundary calculation logic (traverse up through importers)
└── Test: Verify affected module list is accurate on file change
```

- Create `forte/cli/src/module_graph/mod.rs`

#### 1.4 HMR Engine + Client

```
Tasks:
├── Server: file change → Module Graph lookup → send WebSocket payload
├── Client: WebSocket connection + import.meta.hot API
├── Module replacement: fetch new module via dynamic import
├── HMR code injection (add accept/dispose to each module)
└── Test: Verify only the changed module reloads on .tsx modification
```

- Create `forte/cli/src/hmr/engine.rs`
- Create `forte/cli/src/hmr/protocol.rs`
- Create `forte/cli/assets/hmr-client.js`
- Create `forte/cli/src/transform/inject.rs`

#### 1.5 Basic SSR

```
Tasks:
├── Bundle server.tsx → server.js (esbuild)
├── Execute SSR in ski
├── Re-bundle on file change
└── Test: Verify SSR works on page refresh
```

**Phase 1 Completion Criteria:**
- React app runs in browser
- Only the changed module is replaced on .tsx file modification (not full reload)
- However, React component state is reset

---

### Phase 2: React Fast Refresh

**Goal:** Update while preserving React component state

#### 2.1 Enable SWC React Refresh Plugin

```
Tasks:
├── Enable refresh option in swc transform
├── Auto-inject $RefreshReg$, $RefreshSig$ code
├── Generate hook signature tracking code
└── Test: Verify transformed code contains Refresh-related code
```

- Modify `transform/swc.rs`: `refresh_enabled: true`

#### 2.2 React Refresh Runtime Integration

```
Tasks:
├── Bundle react-refresh/runtime (include in pre-bundle)
├── Inject runtime initialization script into HTML
├── Register global functions: $RefreshReg$, $RefreshSig$, $RefreshRuntime$
└── Test: Verify window.$RefreshRuntime$ is accessible in browser console
```

- Create `forte/cli/assets/react-refresh-setup.js`

#### 2.3 HMR + Fast Refresh Integration

```
Tasks:
├── Call performReactRefresh() after module update
├── Component detection logic (Fast Refresh only for React components)
├── Propagate to appropriate boundary on non-component file change
└── Test: Verify useState state is preserved while UI updates
```

- Modify `hmr-client.js`: Add Fast Refresh trigger logic

**Phase 2 Completion Criteria:**
- State preserved when modifying component code
- Appropriate remount on hook changes
- Only related components update on non-component file changes

---

### Phase 3: CSS + Stabilization

**Goal:** Development experience equivalent to Vite

#### 3.1 Tailwind CSS Integration

```
Tasks:
├── Manage tailwindcss CLI process
├── Trigger tailwind rebuild on .tsx file change
├── CSS file change detection + HMR
└── Test: Verify styles are instantly reflected when classes are added
```

- Create `forte/cli/src/css/mod.rs`

#### 3.2 CSS HMR

```
Tasks:
├── Transform CSS imports to dynamic link tags
├── Update link href on CSS change (no reload)
├── Remove existing link on dispose
└── Test: Style changes without page reload on CSS modification
```

#### 3.3 Error Handling + Overlay

```
Tasks:
├── Display overlay in browser on compile error
├── Catch and display runtime errors
├── Auto-recovery after error fix
└── Test: Syntax error → overlay → fix → normal operation
```

#### 3.4 Source Maps

```
Tasks:
├── Generate sourcemap during SWC transform
├── Display original .tsx in browser DevTools
└── Test: Error stack traces show original file:line
```

**Phase 3 Completion Criteria:**
- Tailwind class changes reflected instantly
- Clear error messages on compile errors
- Original source debugging possible in DevTools

---

### Implementation Order Summary

| Phase | Key Deliverable | Verification Method |
|-------|----------------|-------------------|
| **1** | HMR working (state reset) | File edit → only module replaced (not full reload) |
| **2** | Fast Refresh (state preserved) | useState value preserved while UI updates |
| **3** | Complete DX | CSS HMR + error overlay + source maps |

Each Phase produces a working deliverable, making debugging and verification straightforward.

---

## 8. Dependencies

### Rust Crates

```toml
[dependencies]
# SWC (transpilation)
swc_core = { version = "0.90", features = ["ecma_parser", "ecma_transforms", "ecma_transforms_react"] }
swc_ecma_parser = "0.143"
swc_ecma_transforms_react = "0.183"

# File system
notify = "6.0"  # already in use

# WebSocket
tokio-tungstenite = "0.21"  # already in use

# Other
sha2 = "0.10"  # for hashing
```

### NPM Packages

```json
{
  "dependencies": {
    "react-refresh": "^0.14.0"
  },
  "devDependencies": {
    "esbuild": "^0.20.0",
    "tailwindcss": "^4.0.0"
  }
}
```

---

## 9. File Structure

```
forte/
├── cli/
│   ├── src/
│   │   ├── cli/
│   │   │   └── dev.rs           # Modified: HMR Engine integration
│   │   ├── server/
│   │   │   └── mod.rs           # Modified: Transform Pipeline integration
│   │   ├── transform/           # New
│   │   │   ├── mod.rs
│   │   │   ├── swc.rs           # SWC integration
│   │   │   ├── import_rewrite.rs
│   │   │   └── inject.rs        # HMR code injection
│   │   ├── module_graph/        # New
│   │   │   └── mod.rs
│   │   ├── hmr/                 # New (extending existing hmr.rs)
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs
│   │   │   └── protocol.rs
│   │   ├── deps/                # New
│   │   │   └── mod.rs           # Dependency Pre-bundling
│   │   └── css/                 # New
│   │       └── mod.rs           # Tailwind handling
│   └── assets/                  # New
│       ├── hmr-client.js
│       └── react-refresh-setup.js
```

---

## 10. References

- [Vite HMR API](https://vite.dev/guide/api-hmr)
- [React Fast Refresh Implementation Details](https://github.com/pmmmwh/react-refresh-webpack-plugin)
- [SWC React Refresh Plugin](https://swc.rs/docs/configuration/compilation#jsctransformreact)
- [Vite Source Code - HMR](https://github.com/vitejs/vite/blob/main/packages/vite/src/node/server/hmr.ts)
- [Hot Module Replacement is Easy](https://bjornlu.com/blog/hot-module-replacement-is-easy)
