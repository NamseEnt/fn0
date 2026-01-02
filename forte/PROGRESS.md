# Forte CLI 구현 진행 상황

## 완료된 작업

### 1. 프로젝트 구조 리팩토링 ✅

CLI와 서버 로직 분리 완료.

```
forte/src/
├── main.rs           # CLI 진입점 (clap)
├── cli/
│   ├── mod.rs        # 명령어 정의 (dev, init, add, build)
│   ├── dev.rs        # forte dev 구현
│   └── init.rs       # forte init 구현
└── server/
    ├── mod.rs        # SSR 서버 로직
    └── cache.rs      # SimpleCache (WASM/JS 캐싱)
```

### 2. forte init ✅

- 프로젝트 구조 생성 (rs/, fe/)
- Forte.toml, Cargo.toml, package.json 등 설정 파일 생성
- 기본 페이지 템플릿 생성 (index 페이지)
- E2E 테스트 통과 (3개)

### 3. forte dev ✅

- 포트 자동 선택 (3000부터 시작, 사용 중이면 다음 포트)
- `--port` 옵션 지정 시 해당 포트만 시도
- `forte-rs-to-ts` 호출 (Props 타입 생성)
- `cargo build --release` 호출 (백엔드 WASM 빌드)
- `npm run build` 호출 (프론트엔드 빌드)
- SSR 서버 시작 및 요청 처리
- E2E 테스트 통과 (2개)

## 다음 단계

| 순서 | 작업 | 상태 |
|------|------|------|
| 1 | 프론트엔드 라우터 자동 생성 | 대기 |
| 2 | `forte add page` + 테스트 | 대기 |
| 3 | Watch 모드 + 핫 스왑 | 대기 |
| 4 | `forte add action` + 테스트 | 대기 |
| 5 | 정적 에셋 서빙 | 대기 |
| 6 | `forte build` + 테스트 | 대기 |

## 기술적 결정사항

### 의존성 경로
- `forte-rs-to-ts`: `env!("CARGO_MANIFEST_DIR")` 기반 상대 경로
- `forte-json`: path 의존성 (개발 환경)
- `RUSTUP_TOOLCHAIN` 환경변수 제거하여 rust-toolchain.toml 사용

### 프론트엔드 번들링
- rolldown 사용
- `globalThis.handler` 패턴으로 전역 핸들러 노출
- `inlineDynamicImports: true`로 단일 번들 생성

### 백엔드 패키지
- 패키지 이름: `backend` (고정)
- WASM 파일: `rs/target/wasm32-wasip2/release/backend.wasm`
