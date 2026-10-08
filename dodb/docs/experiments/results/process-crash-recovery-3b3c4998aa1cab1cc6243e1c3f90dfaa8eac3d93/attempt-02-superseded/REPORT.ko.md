# 대체된 실행 기록: main-btree와 planned-blink 크래시 복구

> 이 실행은 최종 복구 증거로 채택하지 않는다. 전체 52 case는 통과했지만, 성공 응답 전 중단된 대상 transaction의 기대 revision을 복구된 데이터에서 가져왔다. 독립 기대 상태 조건을 만족하지 않아 보존만 하고, 후속 실행에서 부모가 기록한 prefix revision으로 대상 revision을 계산하도록 고쳤다.

## 판정

계획한 52회 중 52회를 실행해 모두 통과했다. 48회는 자식 프로세스에 SIGKILL을 보낸 크래시 case이고, 4회는 정상 종료 대조군이다. 경계 미도달, timeout, 잘못된 종료 signal, 정상 종료를 크래시 성공으로 세지 않았다. 검증기 자체의 누락 행, 잘못된 값, 잘못된 revision, 부분 transaction 거부 확인도 통과했다.

| 엔진 | 폭 | 대상 성공 반환 후 | before_wal_append | before_wal_sync | after_wal_sync | 정상 종료 |
|---|---:|---:|---:|---:|---:|---:|
| main-btree | 4 | 3/3 | 3/3 | 3/3 | 3/3 | 1/1 |
| main-btree | 8 | 3/3 | 3/3 | 3/3 | 3/3 | 1/1 |
| planned-blink | 4 | 3/3 | 3/3 | 3/3 | 3/3 | 1/1 |
| planned-blink | 8 | 3/3 | 3/3 | 3/3 | 3/3 | 1/1 |

before_wal_append에서는 대상 transaction이 나타나지 않았다. before_wal_sync에서는 12회 모두 완전한 대상 transaction이 보였고, 부분 반영은 없었다. SIGKILL은 커널 page cache를 비우지 않으므로 이 결과는 sync 전 transaction이 사라져야 한다는 증거가 아니다. after_wal_sync hook에서는 12회 모두 hook 도달 handshake를 확인한 뒤 자식을 SIGKILL했고, sync 성공 후 대상 transaction이 복구됐다. 이 경계는 성공 응답 전에 발생했다.

## 실행 범위와 검증 방법

- 각 case는 4096행 seed transaction 뒤 폭 4 또는 8의 성공 transaction 128건을 실행했다. 한 writer가 순차 실행했고 동시에 진행 중인 쓰기는 최대 1개였다.
- 논리 키 크기는 16 bytes(PK 8 bytes와 SK 8 bytes), 값은 64 bytes다. 반복 seed는 `0xd0db2026`, `0xd0db2027`, `0xd0db2028`이다. 대상은 서로 다른 기존 키를 갱신하고, key order에서 여러 leaf에 걸치도록 분산했다. 값은 seed, transaction, mutation 식별자를 담는다.
- 부모는 결정적 입력과 각 seed·prefix 성공 응답 revision으로 독립 기대 상태를 만들었다. 복구된 자식 상태를 정답으로 삼지 않았다. 대상이 덮어쓰는 기존 행은 대상이 반영된 기대 상태에서 새 값과 반환 revision을 검사했다.
- 두 엔진 모두 실제 `ProductionFile`과 `open_with_wal`을 사용했다. Blink는 `enable_planned_execution()`만 활성화했고 parallel execution은 켜지 않았다. 검증은 storage API의 성공 반환 및 재오픈 결과를 다루며 coordinator 클라이언트 종단간 검증은 아니다.
- 경계는 부모·자식 stdout/stdin handshake로 확인했다. 대상 성공 응답 후 경계는 응답을 받은 부모가 대기 중인 해당 자식 PID에 SIGKILL을 보냈다. 나머지 세 경계는 기존 `FaultInjector`가 hook 안에서 부모 신호를 기다릴 때 종료했다. hook 36회 모두 도달했고 SIGKILL 48회는 모두 signal 9로 종료됐다.
- WAL 코드에서 `before_wal_sync`와 `during_wal_sync` hook은 WAL 레코드 기록 뒤 실제 `sync_data()` 호출 전에 실행된다. `after_wal_sync`는 실제 `sync_data()` 성공 직후 실행된다. `during_wal_sync`라는 이름을 syscall 도중으로 해석하거나 그 경계를 이번 테스트 증거로 사용하지 않았다.
- 자식 종료를 기다린 뒤 DB/WAL 복구 전 사본을 저장했다. 새 프로세스로 재오픈해 전체 4096행의 누락·추가 키, 값, revision, 최대 revision, 구조 불변식을 확인했다. 그 뒤 추가 쓰기 한 건의 성공 revision을 기록하고 다시 재오픈해 전체 기대 상태를 재검증했다.

## OCI 출처

- 호스트: `opc@217.142.246.204`; `IdentitiesOnly=yes`, `StrictHostKeyChecking=yes`로 접속했다. `/bench/zfs/db`는 실행 시 `dodbbench/db` ZFS였고 여유 공간은 108 GiB였다.
- 테스트 소스 SHA: `6b5b93fa921225b5acb80d23c32509eb56165dc4`. OCI의 detached checkout은 clean 상태였고 native `aarch64-unknown-linux-gnu` Release로 빌드했다. 실제 checkout toolchain은 `1.97.1-aarch64-unknown-linux-gnu`다. 저장소 Rust 설정에 고정된 toolchain을 사용했다.
- 명령은 `cargo test --release --locked -p dodb-storage --test process_crash_recovery -- --nocapture`다. 이 실행은 테스트 외 성능 측정을 수행하지 않았다.
- 테스트 실행 파일 SHA-256: `600a738f063a07126b5725cbd3c248b9f7fc93e058a42aaf158f75981384df9c`. 자식 helper SHA-256: `829e68361a25c435eee067bafcba399c2e7b486400b46f0561001e09fce90f9b`. helper 해시는 실제 실행 경로와 `deps` 복사본에서 같음을 확인했다. 이 helper 포함 실행 파일을 과거 성능 측정 바이너리라고 표기하지 않는다.
- `95954ecaa`와 비교해 `dodb/crates/dodb-storage/src/btree`, `blink`, `wal.rs`, `durable_file.rs`가 동일했다. 변경은 새 통합 테스트, 자식 helper, 실행 스크립트, 이 결과 기록에 한정했다.
- 실패한 첫 harness 준비 시도는 SHA `5d34c23bd3d594c93419eda66c5177161e56ff24`에서 append 로그의 create 누락으로 테스트 case 진입 전 종료 코드 101을 냈다. 이를 무효 시도로 보존했고 52회 계획에 넣지 않았다. 수정 후 SHA `6b5b93fa921225b5acb80d23c32509eb56165dc4`로 전체 계획을 새 경로에서 다시 실행했다.

## 결과 파일

전체 실행 산출물은 OCI ZFS 경로 `/bench/zfs/db/process-crash-recovery-6b5b93fa921225b5acb80d23c32509eb56165dc4`에 남아 있다. DB/WAL 복구 전 사본 52쌍, case 판정과 handshake/signal/restart 로그, 부모 성공 응답, 기대 최종 상태, test log, 출처 및 바이너리 해시, 376개 파일 체크섬을 보존했다. 전체 체크섬 검증은 376/376 통과했다. 빌드 target과 TMPDIR도 `/bench/zfs/db` 아래의 SHA 전용 경로에 두었다.

같은 OCI의 무효 harness 시도는 `/bench/zfs/db/process-crash-recovery-5d34c23bd3d594c93419eda66c5177161e56ff24`에 보존했다. 이 보고서 디렉터리에는 case 판정표, 성공 응답 기록, 출처, 빌드 toolchain, 실행 및 무효 시도 로그의 사본이 있다. 전체 DB/WAL 및 복구 전 사본의 권위 있는 보존 위치와 체크섬 목록은 OCI ZFS 결과 경로다.

## 확인 범위의 한계

이번 결과는 실제 파일 adapter를 사용한 프로세스 SIGKILL 후 재오픈 복구만 확인한다. 호스트 전원 장애, 커널 page cache 손실, 디스크·장치 손실, 실제 sync syscall 도중의 중단, coordinator를 통한 클라이언트 종단간 동작은 검증하지 않았다. 성능 최적화나 추가 성능 측정도 수행하지 않았다.
