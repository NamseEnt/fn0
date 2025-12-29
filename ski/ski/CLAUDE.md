이 프로젝트는 WinterCG 호환이면서도 nodejs같은 기능은 완전히 배제한 minimum js runtime이다.
module, resolve, import 등은 지원하지 않는다. 대신 rolldown으로 완전히 번들링된 파일을 입력받는다고 가정한다.

## 중요 규칙
- crate는 항상 `cargo add`로 최신 버전을 다운받는다
- 절대로 crate 설치를 위해 Cargo.toml을 직접 수정하지 말 것
- 빌드 체크는 항상 `cargo clippy`를 사용한다 (cargo build 대신)
- 최종 테스트는 `cargo run`으로 실행하여 검증한다
