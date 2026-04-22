# service-style 브랜치

## 주인이 시킨 원래 요구

1. core 하나당 thread 하나 돈다.
2. 그 thread 하나가 여러 요청을 동시에 처리한다.
3. 한 요청이 sub 요청으로 자기 자신을 다시 호출할 수 있다.
4. 자기 자신을 호출할 때는 wasm instance를 **새로 만들지 않고** 원래 instance를 그대로 재이용한다.

초기 상태: 모든 자기 호출이 새 wasm instance에서 처리되고 리턴된다. 이걸 (4)처럼 바꾸는 게 목표.

## 목표 아키텍처 — Cloudflare Workers 모델 (co-location)

JS isolate와 wasm instance 모두 **한 스레드에 pin**하여 공유·재사용한다. 둘 사이는 같은 스레드 내 직접 호출이고, 채널/cross-thread 오버헤드 없음. p3의 `run_concurrent`는 한 instance가 여러 request를 동시에 처리하도록 설계된 API라 wasm 쪽도 이게 가능하다.

**요청 처리 flow**

1. wasm instance의 `service.handle(accessor, req)`를 호출해 응답을 받는다. (모든 요청은 먼저 wasm)
2. 프로젝트가 `WasmJs`이면 1의 응답을 JS isolate의 entry로 주입한다.
3. JS가 실행 중 `fetch`를 하면:
   - URL host == self이면 같은 스레드의 같은 wasmtime instance의 `service.handle(accessor, req)`로 wiring.
   - 그 외면 HTTP pool로 외부 네트워크 요청.
4. JS의 최종 응답이 사용자에게 반환된다. (프로젝트가 `Wasm`이면 1의 응답이 곧 최종 응답)

**핵심 원칙**

- Worker 스레드 수 = core 수. 각 스레드에 `std::thread` + current_thread tokio runtime + `LocalSet`.
- Request dispatch: `hash(code_id) % N` 고정 매핑 (같은 code_id는 같은 스레드 → JS isolate·wasm instance cache locality).
- JS isolate: code_id 당 1개 `Rc<RefCell<JsRuntime>>`, lazy 생성, LRU evict. **Cloudflare Workers와 동일하게 한 isolate에서 여러 request를 single-threaded event loop로 concurrent 처리** (request A가 `await fetch` 동안 request B handler가 같은 isolate에서 진행). `JsRuntime`의 event loop를 장수명으로 계속 돌리고 외부 request마다 `handler(req)` Promise를 isolate에 추가 주입하는 구조.
- Wasm instance: code_id 당 장수명 Store + `run_concurrent` 루프 1개를 계속 열어둠. 외부 request마다 그 루프 안에서 `service.handle(accessor, new_req)`를 추가로 띄움 (이미 self-invoke가 같은 방식). Engine·InstancePre는 전역 `Arc` 공유.
- JS↔wasm 호출은 같은 스레드 내 deno op에서 직접 `service.handle(accessor, ...)` 호출. mpsc 채널 경유 제거.
- 사용자 코드 계약 (JS·wasm 공통): "handler는 stateless, 전역 가변 상태 금지 또는 concurrent-safe". Cloudflare Workers와 동일한 계약. 문서화 필요 (p3·V8 둘 다 이걸 강제하지 않음).
- Fairness: wasmtime epoch interrupt (3ms tick, 기존 유지) + JS watchdog (10ms 초과 시 `isolate.thread_safe_handle().terminate_execution()`).
- 전용 `std::thread` 하나: epoch 증가 + JS watchdog. Worker 부하와 무관하게 tick 보장.

## 진행

- [x] 자기 호출 시 원래 instance 재이용 — **wasm+js 경로**. wasm+js 런타임 모드에서 JS 측이 `/__self_invoke/...`를 부르면 원래 instance의 `service.handle(accessor, ...)`로 재진입. (fn0::js 안에 mpsc 루프 구현)
- [x] 자기 호출 시 원래 instance 재이용 — **wasm만 쓰는 경로**. `SelfInvokeHooks`가 `wasi:http/client.send` 아웃바운드를 가로채 (URI host == incoming Host) 일 때 mpsc로 accessor 루프에 돌려 `service.handle(accessor, ...)`로 재진입. (fn0::self_invoke 모듈, execute.rs·js.rs 공통)

### co-location 마이그레이션 서브태스크

- [x] Worker thread 골격: fn0-worker main이 N개 `std::thread` 생성, 각 스레드가 current_thread tokio runtime + `LocalSet` + request mpsc receiver 운영. (fn0-worker/src/worker_pool.rs)
- [x] Request dispatch: HTTP accept 쪽에서 `hash(code_id) % N` 으로 worker 고르고 envelope를 push. 응답은 oneshot으로 회신. Worker queue 꽉 차면 503. (worker_pool::dispatch + QUEUE_CAPACITY=256)
- [x] 네이밍: `ExecutionContext`(공유, Arc) + `CodeExecutor`(per-thread, !Sync). `Fn0` 삭제.
- [x] Wasm instance를 code_id당 장수명으로: `CodeExecutor.instances: RefCell<HashMap<_, UnboundedSender>>`. 첫 request 시 `spawn_local`로 `execute::run_wasm_instance_loop`. 루프 내부에서 `run_concurrent` 1회 열고 `FuturesUnordered`로 request 들어올 때마다 `service.handle(accessor, req)` concurrent dispatch.
- [x] ski API 재작성: `SkiInstance::load(code, script_path, fetch_handler)` + `call(req) -> Future<Response>` + `drive_forever(self: Rc<Self>)`. id 기반 `SlotMap` (op_take_request_parts(id) / op_respond(id, ...)). `__ski_runHandler(id)` JS side에서 받음. `call_with_args(&run_handler, &[id_v8])`로 concurrent promise. `driver_waker`로 call 시점 이벤트 루프 깨움.
- [x] JS isolate를 code_id당 재사용: `CodeExecutor.js_instances: RefCell<HashMap<_, JsSlot>>`. 첫 WasmJs request 시 `SkiInstance::load` + `spawn_local(driver_instance.drive_forever())`.
- [x] JS↔wasm 직접 호출: `self_invoke::call_wasm_direct(req)`가 thread_local `ACCESSOR_PTR`/`SERVICE_PTR`에서 accessor·service 꺼내 `service.handle` 직접 호출. `WasmForwardingFetchHandler`가 `/__self_invoke/` prefix 매칭해서 이 함수로 route. mpsc 완전 제거.
- [x] Epoch ticker 전용 `std::thread`: `fn0-epoch-ticker` 이름, 3ms `std::thread::sleep` 루프.
- [x] JS 10ms `terminate_execution`: `ski::SkiInstance.deadlines: Arc<Mutex<HashMap<call_id, Instant>>>`. 전역 `WATCHDOG: OnceLock<Mutex<Vec<WatchdogEntry>>>` + 전용 `ski-watchdog` std::thread가 1ms tick하며 만료 deadline 발견 시 `isolate_handle.terminate_execution()`. `JS_CALL_TIMEOUT_MS = 10`.
- [x] forte/cli·cli/local을 LocalSet + CodeExecutor로 마이그레이션: `main` = `Builder::new_current_thread` + `LocalSet::run_until(async_main())`. 모든 connection handler/queue poller가 `spawn_local`. `QueuePoller::run`의 execute_fn Send 요구 제거.
- [x] wasm instance 재사용 계약 문서화: README.md에 "Handler Contract" section 추가.

## 부수 작업 (위 목표 가는 길에 깔린 것들, 완료)

- [x] fn0 wasm 실행을 wasi:http p2 proxy에서 p3 Service + `run_concurrent`로 전환
- [x] wasmtime 42 → 43 업그레이드
- [x] 사용자 wasm이 쓰던 라이브러리(wstd)를 제거하고 forte-sdk를 wasi:http p3 네이티브로 갈아엎음 (runtime/time/rand/http/serve 전부 새로 구현)
- [x] forte-db도 wstd 제거하고 새 forte-sdk로 이식
- [x] 사용자 코드 → wasm 생성기(generate_routes) 재작성 (wasi:http/handler 직접 export)
- [x] `forte init` 스캐폴드 템플릿 교체 (새 SDK 사용)
- [x] wasi:http p3를 쓰는 wasm 테스트 바이너리 러너(forte-test-runner) 작성
- [x] ls-news 디렉토리 git 바깥으로 이동 (`~/ls-news2`, 주인이 지금 관리 안 함)

## 로컬 dev 테스트 계획 — ~/amgi/web

co-location 마이그레이션 전체가 `cargo check`만 통과한 상태. 실제 런타임 동작은 ~/amgi/web 프로젝트를 forte dev로 띄워 검증한다. 이 프로젝트는 WasmJs 배포 (Forte.toml + fn0.toml + rs/ + fe/) 이므로 wasm instance 재사용, JS isolate 재사용, JS↔wasm 직접 호출 모두 한 번에 exercise 된다.

### 빌드·실행 명령어

```sh
# 1) 로컬 forte CLI 재빌드 (cargo install symlink가 /Users/namse/.cargo/bin/forte -> fn0/forte/cli/target/debug/forte로 걸려있음)
cd /Users/namse/fn0/forte/cli && cargo build

# 2) amgi/web 프로젝트로 이동
cd ~/amgi/web

# 3) dev 서버 기동
forte dev
```

포트·프로젝트 루트 별도 지정이 필요하면: `forte dev --project ~/amgi/web --port 8080`.

### 확인 체크리스트 (브라우저 + 서버 로그)

1. **기동 자체 성공**: vite dev 서버 + fn0 worker thread + sqld 모두 뜨고 `Listening on http://localhost:<port>`가 찍힌다.
2. **SSR 페이지 응답**: 페이지 URL 하나 열어 HTML이 정상 렌더링된다 (JS isolate에서 `__ski_runHandler(id)` 호출 → `handler(request)` → `op_respond(id, ...)` 정상 동작 증명).
3. **backend API 호출**: SSR 중 JS가 `/api/...` 또는 `/__self_invoke/...`를 fetch → `WasmForwardingFetchHandler`가 `/__self_invoke/` prefix 매칭 → `self_invoke::call_wasm_direct` → thread_local accessor로 같은 wasm instance 재진입. API JSON 응답이 SSR에 반영되면 성공.
4. **여러 request 연속/동시**: 2-3개 탭에서 같은 페이지 동시에 열어본다. 응답이 섞이거나 panic (borrow_mut 중복, SlotMap id 충돌) 없이 각각 정상 응답.
5. **wasm outbound HTTP**: handler에서 `forte_sdk::http::Client::send`로 self subdomain 부르는 코드가 있으면 `SelfInvokeHooks::send_request`가 `SELF_HOST` task-local과 매칭해 thread_local accessor로 재진입하는지 확인.
6. **소스 수정 → rebuild**: `rs/` 파일 하나 수정하면 dev가 backend wasm 재빌드 + cache invalidate 후 새 instance 뜨는지 (첫 request에서 `spawn_local(run_wasm_instance_loop)` 재실행).
7. **`.env` 수정 반영**: `.env` 바꾸면 `handle.ctx.set_env(DEV_CODE_ID, ...)` 경로 타서 다음 request에 새 env 적용.
8. **JS 10ms watchdog**: handler에 의도적으로 `while(true){}` 무한 루프 하나 넣어 놓고 호출 → 10ms 안에 `terminate_execution`되어 요청이 에러로 끝나는지. (isolate가 terminated 상태로 남아 다음 요청부터 불안정할 수 있음 — 현재 세션 범위 밖 이슈는 기록만 하고 isolate 재생성 로직은 추후.)
9. **epoch interrupt**: wasm에 CPU-bound 1초 루프 넣어 호출 → 1000ms에서 "CPU time limit exceeded" 에러 반환.

### 회귀 의심 포인트 (실패 시 우선 의심할 곳)

- `AccessorGuard` 설치가 run_concurrent closure 시작 시점에 되는가 → self-invoke 시 `call_wasm_direct`가 "no accessor installed" 반환하면 이 지점.
- `SELF_HOST` task_local scope 누락 → SelfInvokeHooks가 self host 판정 못하고 전부 default_send_request (외부 HTTP)로 가는 증상.
- ski `driver_waker` 깨우기 누락 → `SkiInstance::call` 만들어도 event loop가 park 상태로 resolve 안 됨 → 응답 hang.
- `SlotMap.responses.remove(&id)`가 None → JS가 `op_respond(id, ...)`를 안 불렀거나 id mismatch.
- `forte/cli/src/cli/dev.rs`의 queue poller future가 Send 요구 깨진 컴파일 이슈는 `QueuePoller::run`의 `F: Fn -> Pin<Box<dyn Future>>`로 완화했지만 다른 tokio::spawn 호출이 executor 빌리면 !Send로 fail 가능.

### 사용자 실행 후 피드백 필요

실행 결과 (성공/실패, 에러 메시지, 로그)를 공유해 주시면 회귀 있는 부분 수정.

## 외부 이벤트 대기

- [ ] fn0 0.2.15가 crates.io에 발행되면 `fn0-worker/Cargo.toml`의 임시 path dep(`fn0 = { path = "../fn0" }`)을 버전 고정으로 되돌리기
