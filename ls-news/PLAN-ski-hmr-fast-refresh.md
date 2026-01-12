# Ski Dev 환경에서 HMR + React Fast Refresh 구현 계획

## 목표

Vite 없이 Ski를 dev 환경에서도 사용하면서, Vite가 제공하는 HMR(Hot Module Replacement)과 React Fast Refresh 기능을 동일하게 제공한다.

---

## 1. 배경 지식

### 1.1 HMR (Hot Module Replacement)이란?

파일 변경 시 전체 페이지를 새로고침하지 않고, 변경된 모듈만 교체하는 기술.

**동작 흐름:**
1. 파일 시스템 watcher가 파일 변경 감지
2. 서버가 변경된 파일을 모듈 그래프에서 찾음
3. 해당 모듈과 영향받는 모듈들 결정
4. WebSocket으로 클라이언트에 업데이트 지시
5. 클라이언트가 새 모듈 fetch → 기존 모듈 교체

**핵심 개념:**
- **HMR Boundary**: 업데이트를 "수용(accept)"하는 모듈. 이 경계를 기준으로 업데이트 범위가 결정됨
- **Module Graph**: 모듈 간의 import 관계를 추적하는 그래프

### 1.2 React Fast Refresh란?

React 공식 HMR 솔루션. 컴포넌트의 **상태(state)를 유지**하면서 코드를 업데이트.

**동작 원리:**
1. Babel/SWC 플러그인이 코드를 변환하여 컴포넌트 등록 코드 주입
2. `react-refresh/runtime`이 컴포넌트 레지스트리 관리
3. HMR 발생 시 변경된 컴포넌트만 re-render (상태 유지)

**변환 예시:**
```jsx
// 원본 코드
function Counter() {
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount(c => c + 1)}>{count}</button>;
}

// 변환된 코드
var _s = $RefreshSig$();
function Counter() {
  _s();
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount(c => c + 1)}>{count}</button>;
}
_s(Counter, "useState{[count, setCount](0)}");
$RefreshReg$(Counter, "Counter");
```

**제한사항:**
- Class 컴포넌트는 지원 안 됨 (function 컴포넌트 + hooks만)
- `// @refresh reset` 주석으로 강제 remount 가능

---

## 2. 아키텍처 개요

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           FORTE DEV SERVER (Rust)                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────────────────────┐ │
│  │   Watcher    │──▶│  HMR Engine  │──▶│     WebSocket Server         │ │
│  │  (notify)    │   │              │   │  (기존 /__hmr 확장)          │ │
│  └──────────────┘   └──────┬───────┘   └──────────────────────────────┘ │
│                            │                                            │
│                            ▼                                            │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                     Module Graph                                  │   │
│  │  - 모듈 간 import/export 관계 추적                                │   │
│  │  - HMR boundary 정보 관리                                         │   │
│  │  - 각 모듈의 hash (변경 감지용)                                   │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                     Transform Pipeline                            │   │
│  │                                                                   │   │
│  │  .tsx/.ts 파일 요청 시:                                           │   │
│  │  1. TypeScript/JSX 트랜스파일 (esbuild/swc)                       │   │
│  │  2. React Fast Refresh 변환 (swc 플러그인)                        │   │
│  │  3. HMR 클라이언트 코드 주입                                      │   │
│  │  4. Import 경로 rewrite (bare → 실제 경로)                        │   │
│  │                                                                   │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                     Dependency Pre-bundler                        │   │
│  │  - node_modules 의존성 스캔                                       │   │
│  │  - esbuild로 ESM 번들 생성                                        │   │
│  │  - /.forte/deps/ 에 캐시                                          │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                              BROWSER                                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                     HMR Client Runtime                            │   │
│  │  - WebSocket 연결 관리                                            │   │
│  │  - import.meta.hot API 제공                                       │   │
│  │  - 모듈 교체 로직                                                 │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                   React Refresh Runtime                           │   │
│  │  - 컴포넌트 레지스트리 ($RefreshReg$)                             │   │
│  │  - Hook 시그니처 추적 ($RefreshSig$)                              │   │
│  │  - 업데이트 시 performReactRefresh() 호출                         │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 3. 구현해야 할 컴포넌트

### 3.1 서버 사이드 (Rust - forte cli)

#### 3.1.1 Transform Pipeline

**위치:** `forte/cli/src/transform/`

```rust
// transform/mod.rs
pub struct TransformPipeline {
    swc_compiler: SwcCompiler,
    module_graph: Arc<RwLock<ModuleGraph>>,
    dep_cache: Arc<RwLock<DependencyCache>>,
}

impl TransformPipeline {
    /// .tsx/.ts 파일을 브라우저용 JS로 변환
    pub async fn transform(&self, file_path: &Path, is_ssr: bool) -> Result<TransformResult> {
        // 1. 파일 읽기
        // 2. SWC로 트랜스파일 (React Refresh 플러그인 포함)
        // 3. import 경로 rewrite
        // 4. HMR 코드 주입 (SSR이 아닌 경우)
        // 5. 모듈 그래프 업데이트
    }
}

pub struct TransformResult {
    pub code: String,
    pub map: Option<String>,  // sourcemap
    pub deps: Vec<String>,    // import한 모듈들
}
```

#### 3.1.2 Module Graph

**위치:** `forte/cli/src/module_graph/`

```rust
// module_graph/mod.rs
pub struct ModuleGraph {
    modules: HashMap<PathBuf, Module>,
}

pub struct Module {
    pub id: String,              // URL 경로 (예: /src/components/Button.tsx)
    pub file_path: PathBuf,      // 실제 파일 경로
    pub importers: HashSet<String>,  // 이 모듈을 import하는 모듈들
    pub imports: HashSet<String>,    // 이 모듈이 import하는 모듈들
    pub accepts_hmr: bool,           // import.meta.hot.accept() 호출 여부
    pub is_self_accepting: bool,     // 자기 자신을 accept하는지
    pub hash: String,                // 콘텐츠 해시 (변경 감지용)
    pub transform_result: Option<TransformResult>,
}

impl ModuleGraph {
    /// 파일 변경 시 영향받는 모듈들 계산
    pub fn get_hmr_boundaries(&self, changed_file: &Path) -> Vec<HmrUpdate> {
        // 1. 변경된 파일에서 시작
        // 2. importers를 따라 올라가며 탐색
        // 3. accepts_hmr=true인 모듈(boundary)을 찾으면 멈춤
        // 4. boundary까지 도달 못하면 full reload 필요
    }
}

pub struct HmrUpdate {
    pub boundary: String,      // HMR boundary 모듈 ID
    pub modules: Vec<String>,  // 업데이트해야 할 모듈들
    pub full_reload: bool,     // full reload 필요 여부
}
```

#### 3.1.3 HMR Engine

**위치:** `forte/cli/src/hmr/`

```rust
// hmr/mod.rs
pub struct HmrEngine {
    module_graph: Arc<RwLock<ModuleGraph>>,
    broadcaster: HmrBroadcaster,
}

impl HmrEngine {
    /// 파일 변경 처리
    pub async fn handle_file_change(&self, file_path: &Path) -> Result<()> {
        // 1. 모듈 그래프에서 HMR boundary 찾기
        let updates = self.module_graph.read().get_hmr_boundaries(file_path);

        // 2. 변경된 모듈 재변환
        for update in &updates {
            self.retransform_modules(&update.modules).await?;
        }

        // 3. HMR payload 전송
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

**위치:** `forte/cli/src/deps/`

```rust
// deps/mod.rs
pub struct DependencyPrebundler {
    cache_dir: PathBuf,  // .forte/deps/
}

impl DependencyPrebundler {
    /// node_modules 의존성을 ESM으로 번들링
    pub async fn prebundle(&self, project_root: &Path) -> Result<DependencyMap> {
        // 1. package.json에서 dependencies 읽기
        // 2. 실제 사용되는 의존성 스캔 (import 분석)
        // 3. esbuild로 각 의존성 번들링
        //    - CommonJS → ESM 변환
        //    - 내부 imports 번들링 (react 내부의 여러 파일들)
        // 4. .forte/deps/react.js, .forte/deps/react-dom.js 등 생성
        // 5. import map 반환
    }
}

pub struct DependencyMap {
    pub entries: HashMap<String, String>,  // "react" → "/.forte/deps/react-xxx.js"
}
```

#### 3.1.5 SWC Integration

**위치:** `forte/cli/src/transform/swc.rs`

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
        // SWC 설정
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

        // 변환 수행
        // 1. TypeScript → JavaScript
        // 2. JSX → React.createElement (또는 jsx-runtime)
        // 3. React Refresh 코드 주입
    }
}
```

### 3.2 클라이언트 사이드 (JavaScript)

#### 3.2.1 HMR Client Runtime

**위치:** `forte/cli/assets/hmr-client.js`

```javascript
// hmr-client.js
// HTML에 주입되어 브라우저에서 실행됨

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
      // 재연결 로직
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

      // 새 모듈 fetch
      for (const moduleId of update.modules) {
        const newUrl = `${moduleId}?t=${payload.timestamp}`;
        const newModule = await import(newUrl);

        // 등록된 콜백 실행
        const entry = this.moduleMap.get(moduleId);
        if (entry && entry.callbacks.length > 0) {
          for (const cb of entry.callbacks) {
            cb(newModule);
          }
        }
      }

      // React Refresh 트리거
      if (window.$RefreshRuntime$) {
        window.$RefreshRuntime$.performReactRefresh();
      }
    }
  }

  // import.meta.hot API 구현
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
        // 모듈 제거 시 실행할 cleanup
        this.registerDispose(moduleId, callback);
      },

      invalidate: () => {
        // 강제로 상위로 전파
        this.socket.send(JSON.stringify({ type: 'invalidate', path: moduleId }));
      },

      data: {},  // HMR 간에 데이터 전달용
    };

    return hot;
  }
}

// 전역 인스턴스
window.__forte_hmr = new HMRClient();

// import.meta.hot polyfill
// (각 모듈에 주입되는 코드에서 사용)
window.__forte_createHotContext = (moduleId) => {
  return window.__forte_hmr.createHotContext(moduleId);
};
```

#### 3.2.2 React Refresh Runtime Wrapper

**위치:** `forte/cli/assets/react-refresh-runtime.js`

```javascript
// react-refresh-runtime.js
// react-refresh/runtime을 wrapping

import RefreshRuntime from 'react-refresh/runtime';

// React Refresh 초기화
RefreshRuntime.injectIntoGlobalHook(window);

// 전역 함수 등록 (변환된 코드에서 사용)
window.$RefreshReg$ = (type, id) => {
  RefreshRuntime.register(type, window.__currentModuleId + ' ' + id);
};

window.$RefreshSig$ = RefreshRuntime.createSignatureFunctionForTransform;

// HMR 업데이트 후 호출
window.$RefreshRuntime$ = RefreshRuntime;
```

#### 3.2.3 각 모듈에 주입되는 HMR 코드

**변환 시 각 .tsx 파일에 자동 주입:**

```javascript
// 모듈 시작 부분에 주입
import.meta.hot = window.__forte_createHotContext("/src/components/Button.tsx");
window.__currentModuleId = "/src/components/Button.tsx";

var $RefreshReg$ = window.$RefreshReg$;
var $RefreshSig$ = window.$RefreshSig$;

// ... 원본 코드 (React Refresh 변환 적용됨) ...

// 모듈 끝 부분에 주입
if (import.meta.hot) {
  import.meta.hot.accept();  // self-accepting으로 등록

  // React Refresh 스케줄링
  window.$RefreshRuntime$.performReactRefresh();
}
```

---

## 4. Import 경로 Rewrite

### 4.1 처리해야 할 케이스

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

### 4.2 구현 로직

```rust
// transform/import_rewrite.rs
pub struct ImportRewriter {
    project_root: PathBuf,
    dependency_map: DependencyMap,
    timestamp: u64,
}

impl ImportRewriter {
    pub fn rewrite(&self, source: &str, current_file: &Path) -> String {
        // 1. 모든 import 문 파싱
        // 2. 각 import의 specifier 분류:
        //    - bare: 'react', 'react-dom/client'
        //    - relative: './foo', '../bar'
        //    - alias: '@/lib/utils'
        // 3. 분류에 따라 경로 변환
        // 4. timestamp 쿼리 파라미터 추가 (캐시 무효화용)
    }
}
```

---

## 5. CSS 처리

### 5.1 Tailwind CSS

```rust
// css/tailwind.rs
pub struct TailwindProcessor {
    config_path: PathBuf,
    output_path: PathBuf,
}

impl TailwindProcessor {
    pub async fn build(&self) -> Result<()> {
        // tailwindcss CLI 실행
        // npx tailwindcss -i ./src/index.css -o ./.forte/css/main.css --watch
    }

    pub async fn rebuild_on_change(&self, changed_file: &Path) -> Result<bool> {
        // .tsx/.html 파일 변경 시 tailwind 클래스가 바뀌었을 수 있음
        // 재빌드 후 HMR 전송
    }
}
```

### 5.2 CSS HMR

```javascript
// CSS 파일 import 시 변환
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

## 6. SSR 처리

### 6.1 SSR용 변환 (Fast Refresh 제외)

```rust
impl TransformPipeline {
    pub async fn transform_for_ssr(&self, file_path: &Path) -> Result<TransformResult> {
        // SSR에서는 HMR/Fast Refresh 불필요
        // 순수 트랜스파일만 수행
        // - TypeScript → JavaScript
        // - JSX → React.createElement
        // - import 경로 rewrite (node_modules는 그대로)
    }
}
```

### 6.2 SSR 모듈 로딩

현재: `vite.ssrLoadModule()`
대체: `ski`에서 직접 번들된 server.js 실행

```rust
// server/ssr.rs
pub struct SsrRenderer {
    ski_runtime: SkiRuntime,
    server_bundle_path: PathBuf,
}

impl SsrRenderer {
    pub async fn render(&self, url: &str, props: Value) -> Result<String> {
        // 1. server.js가 변경되었으면 재번들링
        // 2. ski로 실행
        // 3. HTML 반환
    }

    async fn rebuild_server_bundle(&self) -> Result<()> {
        // esbuild로 server.tsx → server.js 번들링
        // node_modules는 external
    }
}
```

---

## 7. 단계별 구현 계획

### Phase 1: 기본 인프라 (1단계)

**목표:** Vite 없이 .tsx 파일 서빙

1. **SWC 통합**
   - `swc_core` crate 추가
   - 기본 TypeScript/JSX 트랜스파일 구현
   - 테스트: 단일 .tsx 파일 변환

2. **기본 파일 서버**
   - `/src/**/*.tsx` 요청 처리
   - 트랜스파일 + 응답
   - 캐시 구현

3. **의존성 Pre-bundling**
   - esbuild로 node_modules 번들링
   - `/.forte/deps/` 서빙
   - import rewrite 구현

**완료 조건:** 브라우저에서 React 앱이 (새로고침으로) 동작

### Phase 2: 모듈 그래프 + HMR (2단계)

**목표:** 파일 변경 시 해당 모듈만 교체

1. **Module Graph 구현**
   - 모듈 간 관계 추적
   - 변환 시 import 분석
   - HMR boundary 계산

2. **HMR Client 구현**
   - WebSocket 연결
   - `import.meta.hot` API
   - 모듈 교체 로직

3. **HMR Engine 구현**
   - 파일 변경 감지 연동
   - HMR payload 생성
   - WebSocket 브로드캐스트

**완료 조건:** 파일 변경 시 해당 모듈만 다시 로드 (상태는 초기화됨)

### Phase 3: React Fast Refresh (3단계)

**목표:** React 컴포넌트 상태 유지

1. **SWC React Refresh 플러그인 활성화**
   - `$RefreshReg$`, `$RefreshSig$` 코드 주입
   - Hook 시그니처 추적

2. **React Refresh Runtime 통합**
   - `react-refresh/runtime` 번들링
   - 전역 함수 등록
   - HMR 업데이트 시 `performReactRefresh()` 호출

3. **모듈별 HMR 코드 주입**
   - 각 React 컴포넌트 파일에 accept/dispose 코드 주입
   - 컴포넌트 감지 로직

**완료 조건:** 컴포넌트 코드 수정 시 상태 유지되며 업데이트

### Phase 4: CSS + 안정화 (4단계)

**목표:** 완전한 dev 경험

1. **Tailwind CSS 통합**
   - tailwindcss CLI 실행
   - 파일 변경 시 재빌드
   - CSS HMR

2. **SSR 지원**
   - server.tsx 번들링
   - ski로 SSR 실행
   - SSR 코드 변경 시 재번들링

3. **에러 처리**
   - 컴파일 에러 overlay
   - 런타임 에러 표시
   - 에러 복구

4. **소스맵**
   - 변환 시 소스맵 생성
   - 브라우저 DevTools 지원

**완료 조건:** Vite와 동등한 DX

---

## 8. 의존성

### Rust Crates

```toml
[dependencies]
# SWC (트랜스파일)
swc_core = { version = "0.90", features = ["ecma_parser", "ecma_transforms", "ecma_transforms_react"] }
swc_ecma_parser = "0.143"
swc_ecma_transforms_react = "0.183"

# 파일 시스템
notify = "6.0"  # 이미 사용 중

# WebSocket
tokio-tungstenite = "0.21"  # 이미 사용 중

# 기타
sha2 = "0.10"  # 해시용
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

## 9. 파일 구조

```
forte/
├── cli/
│   ├── src/
│   │   ├── cli/
│   │   │   └── dev.rs           # 수정: HMR Engine 통합
│   │   ├── server/
│   │   │   └── mod.rs           # 수정: Transform Pipeline 통합
│   │   ├── transform/           # 신규
│   │   │   ├── mod.rs
│   │   │   ├── swc.rs           # SWC 통합
│   │   │   ├── import_rewrite.rs
│   │   │   └── inject.rs        # HMR 코드 주입
│   │   ├── module_graph/        # 신규
│   │   │   └── mod.rs
│   │   ├── hmr/                 # 신규 (기존 hmr.rs 확장)
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs
│   │   │   └── protocol.rs
│   │   ├── deps/                # 신규
│   │   │   └── mod.rs           # 의존성 Pre-bundling
│   │   └── css/                 # 신규
│   │       └── mod.rs           # Tailwind 처리
│   └── assets/                  # 신규
│       ├── hmr-client.js
│       └── react-refresh-setup.js
```

---

## 10. 참고 자료

- [Vite HMR API](https://vite.dev/guide/api-hmr)
- [React Fast Refresh 구현 상세](https://github.com/pmmmwh/react-refresh-webpack-plugin)
- [SWC React Refresh 플러그인](https://swc.rs/docs/configuration/compilation#jsctransformreact)
- [Vite 소스코드 - HMR](https://github.com/vitejs/vite/blob/main/packages/vite/src/node/server/hmr.ts)
- [Hot Module Replacement is Easy](https://bjornlu.com/blog/hot-module-replacement-is-easy)
