# Fetch Handler & Hooks System 구현 계획

## 배경

현재 forte는 페이지 단위로 Props를 fetch한 후 렌더링하는 방식이다. 이 방식은 모든 페이지에서 공통 데이터(예: 로그인 유저 정보)를 반복해서 Props에 넣어야 하는 문제가 있다.

이를 해결하기 위해 React Suspense 기반의 hook 시스템을 도입한다. Rust에서 hook을 정의하면 프론트엔드에서 `useXxx()` 형태로 사용할 수 있게 한다.

## 핵심 아이디어

1. **ski에 범용 fetch handler 추가**: ski가 forte 전용이 되지 않도록, 모든 fetch 요청을 가로챌 수 있는 범용 인터페이스 제공
2. **forte가 hook 패턴 정의**: forte가 `/__forte_hook/*` URL을 인터셉트하여 WASM hook 호출
3. **Suspense로 async 처리**: hook 호출은 async이므로 React Suspense로 처리

---

## Phase 1: ski - 범용 Fetch Handler

### 목표
ski가 범용 JS runtime으로 유지되면서, 사용자가 fetch 동작을 커스터마이즈할 수 있게 한다.

### 1-1. Rust 인터페이스 (`ski/ski/src/lib.rs`)

```rust
use std::future::Future;
use std::pin::Pin;

pub type FetchHandlerFuture = Pin<Box<dyn Future<Output = Option<Response>> + Send>>;

pub trait FetchHandler: Send + Sync + 'static {
    /// fetch 요청을 가로챌 수 있다.
    /// - Some(Response): 가로채서 이 응답 사용
    /// - None: 원래 fetch 동작 수행
    fn handle(&self, request: Request) -> FetchHandlerFuture;
}

/// run() 시그니처 변경
pub async fn run(
    code: &str,
    request: Request,
    fetch_handler: Option<Arc<dyn FetchHandler>>,
) -> Result<Response>
```

### 1-2. Op 추가 (`ski/ski/src/runtime_options.rs`)

```rust
#[op2(async)]
async fn op_fetch_intercept(
    state: Rc<RefCell<OpState>>,
    #[string] url: String,
    #[string] method: String,
    #[serde] headers: Vec<(String, String)>,
    #[serde] body: Option<Vec<u8>>,
) -> Result<Option<FetchInterceptResult>, JsErrorBox> {
    // OpState에서 FetchHandler 가져와서 호출
    // None이면 JS에서 원래 fetch 수행
}

#[derive(Serialize)]
struct FetchInterceptResult {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}
```

### 1-3. JavaScript 수정 (`ski/ski/bootstrap.js`)

```js
const originalFetch = fetch.fetch;

async function interceptedFetch(input, init) {
  const url = typeof input === "string" ? input : input.url;
  const method = init?.method || "GET";
  const headers = init?.headers ? Array.from(new Headers(init.headers).entries()) : [];
  const body = init?.body || null;

  // Rust handler에게 먼저 물어봄
  const intercepted = await core.ops.op_fetch_intercept(url, method, headers, body);

  if (intercepted !== null) {
    // Handler가 처리함
    return new Response(new Uint8Array(intercepted.body), {
      status: intercepted.status,
      headers: intercepted.headers,
    });
  }

  // 원래 fetch 수행
  return originalFetch(input, init);
}

Object.defineProperty(globalThis, "fetch", {
  value: interceptedFetch,
  enumerable: true,
  configurable: true,
  writable: true,
});
```

---

## Phase 2: forte - Hook 시스템

### 목표
forte가 ski의 fetch handler를 활용하여 `/__forte_hook/*` 요청을 WASM hook으로 처리한다.

### 2-1. ForteFetchHandler 구현 (`forte/cli/src/server/`)

```rust
pub struct ForteFetchHandler {
    wasm_runner: Arc<WasmRunner>,
    request_context: RequestContext,  // cookies 등
}

impl FetchHandler for ForteFetchHandler {
    fn handle(&self, req: Request) -> FetchHandlerFuture {
        let path = req.uri().path();

        if !path.starts_with("/__forte_hook/") {
            return Box::pin(async { None });
        }

        let hook_name = path.strip_prefix("/__forte_hook/").unwrap();
        let wasm_runner = self.wasm_runner.clone();
        let cookies = self.request_context.cookies.clone();

        Box::pin(async move {
            let result = wasm_runner
                .call_hook(hook_name, cookies, req.body())
                .await;

            match result {
                Ok(output) => Some(Response::builder()
                    .status(200)
                    .header("content-type", "application/json")
                    .body(output)
                    .unwrap()),
                Err(e) => Some(Response::builder()
                    .status(500)
                    .body(e.to_string())
                    .unwrap()),
            }
        })
    }
}
```

### 2-2. Hook 정의 규약 (`rs/src/hooks/*.rs`)

```rust
// rs/src/hooks/me.rs
use cookie::CookieJar;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {}

#[derive(Serialize)]
pub struct Output {
    pub user: Option<User>,
    pub github_auth_url: String,
}

#[derive(Serialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub avatar_url: String,
}

pub fn handler(jar: &CookieJar, _input: Input) -> Output {
    let me = crate::common::auth::get_me(jar);
    let github_auth_url = crate::common::auth::create_github_auth_url(
        jar,
        "", // origin은 실제로는 context에서 가져와야 함
        crate::route_generated::Redirect::Index,
    );

    Output {
        user: me.map(|u| User {
            id: u.user_id,
            username: u.username,
            avatar_url: u.avatar_url,
        }),
        github_auth_url,
    }
}
```

### 2-3. Hook Codegen

`rs/src/hooks/me.rs` → `fe/src/hooks/useMe.ts` 자동 생성:

```typescript
// Auto-generated from rs/src/hooks/me.rs
import { z } from "zod";
import { useForteHook } from "@forte/react";

const InputSchema = z.object({});
type Input = z.infer<typeof InputSchema>;

const UserSchema = z.object({
  id: z.string(),
  username: z.string(),
  avatarUrl: z.string(),
});

const OutputSchema = z.object({
  user: UserSchema.nullable(),
  githubAuthUrl: z.string(),
});
type Output = z.infer<typeof OutputSchema>;

export function useMe(input: Input = {}): Output {
  return useForteHook("me", input, OutputSchema);
}
```

### 2-4. React Runtime (`@forte/react`)

```typescript
// fe/src/lib/forte-react.ts

const hookCache = new Map<string, { promise: Promise<any>; result?: any }>();

export function useForteHook<T>(
  hookName: string,
  input: any,
  schema: z.ZodSchema<T>
): T {
  const cacheKey = `${hookName}:${JSON.stringify(input)}`;

  const cached = hookCache.get(cacheKey);

  if (cached?.result !== undefined) {
    return cached.result;
  }

  if (cached?.promise) {
    throw cached.promise;  // Suspense trigger
  }

  const promise = fetch(`/__forte_hook/${hookName}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  })
    .then(res => res.json())
    .then(data => {
      const result = schema.parse(data);
      hookCache.set(cacheKey, { promise, result });
      return result;
    });

  hookCache.set(cacheKey, { promise });
  throw promise;  // Suspense trigger
}
```

---

## Phase 3: ls-news 적용

### 3-1. Hook 작성

```
rs/src/hooks/
  mod.rs
  me.rs
```

### 3-2. Frontend 적용

**Root Suspense (`fe/src/client.tsx`, `fe/src/server.tsx`):**
```tsx
<Suspense fallback={<div>Loading...</div>}>
  <App />
</Suspense>
```

**NewsHeader에서 사용:**
```tsx
import { useMe } from "@/hooks/useMe";

export function NewsHeader() {
  const { user, githubAuthUrl } = useMe();

  return (
    <header>
      {user ? (
        <div>
          <img src={user.avatarUrl} />
          <span>{user.username}</span>
          <button onClick={signOut}>ログアウト</button>
        </div>
      ) : (
        <a href={githubAuthUrl}>GitHubでログイン</a>
      )}
    </header>
  );
}
```

---

## 구현 순서

1. **Phase 1-1**: ski에 `FetchHandler` trait 추가
2. **Phase 1-2**: ski에 `op_fetch_intercept` op 추가
3. **Phase 1-3**: ski bootstrap.js에서 fetch 감싸기
4. **Phase 2-1**: forte에 `ForteFetchHandler` 구현
5. **Phase 2-2**: hooks 폴더 구조 및 규약 정의
6. **Phase 2-3**: hook codegen (forte-rs-to-ts 확장)
7. **Phase 2-4**: `@forte/react` runtime 구현
8. **Phase 3**: ls-news에 `useMe` hook 적용 및 테스트

---

## 고려사항

### SSR에서의 동작
- SSR 시에도 `fetch`가 호출되므로, ski 내부에서 hook이 처리됨
- Hydration 시 클라이언트에서도 같은 hook 호출 (캐시 활용 가능)

### 캐싱 전략
- 같은 request 내에서 동일 hook 여러 번 호출 시 캐시
- 캐시 무효화 전략 필요 (로그아웃 시 등)

### 에러 처리
- Hook 실행 실패 시 Error Boundary로 처리
- 또는 Output에 error 필드 포함

### Origin 문제
- `create_github_auth_url`에 origin 필요
- Request context에서 origin 추출하여 hook에 전달
