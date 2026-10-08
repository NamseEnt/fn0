# main-btree와 planned-blink 프로세스 크래시 복구 보고서

## 최종 판정

유효한 최종 실행은 계획 52회, 실제 52회, 통과 52회, 실패 0회, 미도달 0회다. 48회는 자식 프로세스 SIGKILL 크래시 case이고, 4회는 정상 종료 대조군이다. case마다 별도 DB/WAL을 사용했다.

| 엔진 | 폭 | 성공 반환 후 | before_wal_append | before_wal_sync | after_wal_sync | 정상 종료 |
|---|---:|---:|---:|---:|---:|---:|
| main-btree | 4 | 3/3 | 3/3 | 3/3 | 3/3 | 1/1 |
| main-btree | 8 | 3/3 | 3/3 | 3/3 | 3/3 | 1/1 |
| planned-blink | 4 | 3/3 | 3/3 | 3/3 | 3/3 | 1/1 |
| planned-blink | 8 | 3/3 | 3/3 | 3/3 | 3/3 | 1/1 |

36개 fault hook handshake를 확인했고, 크래시 case 48개는 모두 자식 프로세스의 종료 signal이 SIGKILL(9)이었다. 경계 미도달, timeout, 정상 종료는 크래시 성공으로 세지 않았다. 검증기 자체가 누락 키, 잘못된 값, 잘못된 revision, 부분 transaction을 거부하는 self-check도 통과했다.

before_wal_append에서는 12회 모두 대상 transaction이 보이지 않았다. before_wal_sync에서는 허용된 두 상태 중 전체 대상 반영이 12회 관찰됐고 부분 반영은 없었다. SIGKILL은 커널 page cache를 지우지 않으므로, 이 관찰은 sync 전 transaction이 사라져야 한다는 뜻이 아니다. after_wal_sync hook은 12회 모두 도달했고, 실제 `sync_data()` 성공 뒤 자식을 종료한 후 전체 대상 transaction이 복구됐다. 이 hook은 대상 성공 응답보다 먼저 발생했다.

## 조건과 독립 기대 상태

- 각 case에서 4096행을 seed한 다음 폭 4 또는 8의 성공 transaction 128건을 수행했다. 단일 writer이며 동시에 진행 중인 쓰기는 최대 1개였다. 반복 seed는 `0xd0db2026`, `0xd0db2027`, `0xd0db2028`이다.
- 논리 키는 PK 8 bytes와 SK 8 bytes로 구성된 16 bytes이고 값은 64 bytes다. 모든 값은 seed, transaction, mutation 식별자를 담는다. 대상 transaction은 서로 다른 seed 행을 갱신하며, key order의 서로 떨어진 위치에 배치해 여러 leaf를 지나도록 했다.
- 부모는 seed·prefix의 각 성공 응답 revision을 기록하고 전체 prefix 상태를 독립 계산했다. 반환 전 경계의 대상 revision은 복구된 상태에서 읽지 않았다. 부모가 각 case의 prefix DB/WAL을 별도 경로에 복제한 뒤, 같은 엔진 모드와 실제 `ProductionFile`로 oracle 대상 transaction을 성공시켜 반환 revision을 얻었다. 부모는 그 oracle revision으로 대상 기대 상태를 자식 실행 전에 기록했다. 자식 결과는 이 기대 상태와 비교했다.
- 폭이 있는 transaction에서 엔진별 revision 증가량이 단순히 1씩 진행된다고 가정하지 않았다. 실제 자식의 성공 반환 revision은 부모 oracle 반환 revision과 비교했고, 반환 전 중단 case도 같은 oracle 기대 revision과 값 전체를 검사했다.
- 두 엔진 모두 실제 `ProductionFile`과 `open_with_wal`을 사용했다. Blink는 `enable_planned_execution()`만 켰고 parallel execution은 켜지 않았다. 이 결과는 storage API 성공 반환 및 재오픈 복구 검증이며 coordinator 클라이언트 종단간 검증이 아니다.
- 부모와 자식은 stdout/stdin handshake로 지정 경계 도달을 확인했다. 성공 반환 경계에서는 부모가 성공 응답과 revision을 받은 다음 자식이 다른 작업이나 정상 종료를 하지 않도록 대기시켰다. 나머지 세 경계는 기존 `FaultInjector` 안에서 부모의 종료 신호를 기다렸다.
- WAL 코드에서 `before_wal_sync`와 `during_wal_sync`는 WAL 레코드 기록 뒤 실제 `sync_data()` 호출 전에 실행된다. `after_wal_sync`는 `sync_data()` 성공 직후 실행된다. hook 이름만으로 syscall 실행 도중이라고 해석하지 않았고 `during_wal_sync`는 이번 경계로 사용하지 않았다.
- 자식 종료를 기다린 뒤 복구 전 DB/WAL 사본을 보존했다. 새 프로세스로 재오픈해 전체 키 집합, 누락·예상 밖 키, 값, revision, 최대 revision, 구조 불변식을 검사했다. 복구 뒤 추가 transaction 한 건을 성공시키고 revision을 기록한 다음 다시 열어 전체 상태를 확인했다.

## OCI와 빌드 출처

- 지정 호스트 `opc@217.142.246.204`를 `IdentitiesOnly=yes`, `StrictHostKeyChecking=yes`로 사용했다. `/bench/zfs/db`는 실행 시 `dodbbench/db` ZFS였으며, 모든 DB/WAL, oracle, 복구 전 사본, Cargo target, TMPDIR과 결과를 이 파일 시스템 아래의 SHA 전용 경로에 두었다.
- 테스트 소스 SHA는 `3b3c4998aa1cab1cc6243e1c3f90dfaa8eac3d93`이다. OCI는 이 SHA의 clean detached checkout에서 native `aarch64-unknown-linux-gnu` Release로 빌드했다. 실제 toolchain은 `1.97.1-aarch64-unknown-linux-gnu`이며 저장소 toolchain 설정에서 선택됐다.
- 실행 명령은 `cargo test --release --locked -p dodb-storage --test process_crash_recovery -- --nocapture`다. 테스트 실행 결과는 1개 통합 테스트 통과, 0 실패이고 추가 성능 측정은 하지 않았다.
- 테스트 실행 파일 SHA-256은 `72747a60768871a37d0677c593e65c52b11688976e042ce9feb0c97815076c01`이다. 실제 자식 helper SHA-256은 `bef83efd7be65eb8ba75dca27616a1561c1d19e8752297a1667ef550953e16da`이며 실행 경로와 Cargo `deps` 사본이 일치했다. 새 helper가 포함된 바이너리를 과거 성능 바이너리라고 표기하지 않는다.
- 앞선 성능 측정 소스 `95954ecaa`와 비교해 `dodb/crates/dodb-storage/src/btree`, `blink`, `wal.rs`, `durable_file.rs`가 동일했다. 엔진, WAL, coordinator, phase0-bench 구현과 제품 정책은 변경하지 않았다.

## 대체된 시도 기록

최종 52회에 포함하지 않은 세 시도를 보존했다.

1. SHA `5d34c23bd3d594c93419eda66c5177161e56ff24`: 로그 파일 생성 옵션 누락으로 테스트 case에 진입하기 전에 종료 코드 101을 냈다. 실행 case는 0회다.
2. SHA `6b5b93fa921225b5acb80d23c32509eb56165dc4`: 52회가 통과했지만 반환 전 경계의 대상 revision을 복구된 상태에서 읽어 기대 revision을 만들었다. 독립 기대 상태가 아니므로 최종 증거에서 제외했다.
3. SHA `a3b554191e6c003fd723d4f2b9ade886cbd1d5ee`: 52 case를 실행했으나 부모가 prefix 최대 revision에 1을 더하는 잘못된 산식으로 대상 revision을 판정해 40건이 verifier failure로 기록됐다. 엔진 실패 증거가 아니며, 이 원본을 그대로 보존했다. 이후 부모 ProductionFile oracle transaction의 반환 revision으로 기대값을 계산했다.

## 산출물과 한계

주 실행 전체 산출물은 OCI ZFS의 `/bench/zfs/db/process-crash-recovery-3b3c4998aa1cab1cc6243e1c3f90dfaa8eac3d93`에 남아 있다. 52개 case 판정, hook/signal/restart 로그, 성공 응답 6,828건, case별 독립 prefix/target 기대 상태, oracle DB/WAL, 복구 전 DB/WAL 사본, test log와 출처를 보존했다. 전체 체크섬 manifest 637개 항목을 전부 검증했다. 결과는 약 803 MB다.

이 보고서 디렉터리에는 52행 판정표, 성공 응답 기록, 빌드·바이너리 출처, OCI 체크섬 목록, 실행 로그 및 대체된 시도 기록을 둔다. 큰 기대 상태 파일과 DB/WAL 사본은 OCI 경로에서 확인한다.

이번 결과는 실제 파일 adapter를 이용한 프로세스 SIGKILL 후 재오픈 복구만 확인한다. 호스트 전원 장애, 커널 page cache 손실, 디스크·장치 손실, 실제 sync syscall 도중 중단, coordinator 클라이언트 종단간 동작은 검증하지 않았다. 성능 최적화와 추가 성능 측정도 수행하지 않았다.
