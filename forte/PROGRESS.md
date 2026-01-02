# Forte CLI 구현 진행 상황

## 완료된 작업

### 1. 프로젝트 구조 리팩토링 ✅

CLI와 서버 로직 분리 완료.

```
forte/src/
├── main.rs           # CLI 진입점 (clap)
├── cli/
│   ├── mod.rs        # 명령어 정의 (dev, init, add, build)
│   └── dev.rs        # forte dev 구현
└── server/
    ├── mod.rs        # SSR 서버 로직
    └── cache.rs      # SimpleCache (WASM/JS 캐싱)
```

### 2. forte dev 기본 구현 ✅

- 포트 자동 선택 (3000부터 시작, 사용 중이면 다음 포트)
- `--port` 옵션 지정 시 해당 포트만 시도, 실패하면 exit(1)
- SSR 서버 시작 및 요청 처리
- `forte-rs-to-ts` 호출 (codegen)
- `cargo build --release` 호출 (백엔드 빌드)
- `npm run build` 호출 (프론트엔드 빌드)
- `forte-manual` 프로젝트에서 수동 테스트 통과

## 다음 단계

### E2E 테스트 기반 개발로 전환

수동 테스트 → 자동화된 E2E 테스트로 전환.

| 순서 | 작업 | 상태 |
|------|------|------|
| 1 | E2E 테스트 인프라 구축 | 대기 |
| 2 | `forte init` 구현 + 테스트 | 대기 |
| 3 | `forte dev` E2E 테스트 | 대기 |
| 4 | 프론트엔드 라우터 자동 생성 | 대기 |
| 5 | `forte add page` + 테스트 | 대기 |
| 6 | Watch 모드 + 핫 스왑 | 대기 |

### 즉시 해야 할 일

1. **E2E 테스트 의존성 추가** (`Cargo.toml`)
2. **`forte init` 테스트 코드 작성**
3. **`forte init` 구현** (테스트 통과하도록)
4. **`forte dev` E2E 테스트 작성** (init으로 생성한 프로젝트 사용)

## 임시 해결 (나중에 수정 필요)

- `forte-rs-to-ts`: `cargo run`으로 호출 (하드코딩된 경로)
- TODO: cargo binstall (github releases) 전환 예정
- TODO: `RUSTUP_TOOLCHAIN` 환경변수 설정 방식으로 변경
