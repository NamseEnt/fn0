# OCI A1 planned-blink 120초 측정

## 결과 요약

지정 OCI에서 `95954ecaaae94757cc3705bc2aacf8c6bb78915f`의 동일 바이너리로 Get, Query16, 혼합 폭 4·8을 각각 비교했다. 유효한 측정은 8/8회다. 첫 네 번의 읽기 실행과 마지막 네 번의 혼합 실행은 순서대로 직렬 수행했다. 각 실행은 별도 데이터베이스 경로에서 시작했다.

| workload | main-btree | planned-blink | planned/main |
|---|---:|---:|---:|
| Get, read/s | 1,252,767 | 1,228,567 | 0.981x |
| Query16, query/s | 211,834 | 191,147 | 0.902x |
| mixed 50/50, 폭 4, aggregate ops/s | 1,461.3 | 2,241.3 | 1.534x |
| mixed 50/50, 폭 8, aggregate ops/s | 785.7 | 2,085.1 | 2.654x |

혼합 처리량은 read 요청과 write transaction을 각각 한 operation으로 합산했다. 읽기·쓰기 transaction·mutation 처리량은 아래 표와 `per-run.tsv`에 따로 기록했다. 반복은 조건별 1회이므로 통계적 안정성이나 일반적인 성능 우위를 확정하지 않는다.

## 측정 조건과 출처

- 실행 호스트는 OCI A1 `opc@217.142.246.204`, ARM64 Oracle Linux 9.8, 2 OCPU다. SSH는 `IdentitiesOnly=yes`와 기본 host-key 검증을 사용했다. `findmnt` 결과 DB, WAL, `TMPDIR`, 기존 build target 모두 `dodbbench/db` ZFS(`/bench/zfs/db`) 아래였다. pool 속성은 `recordsize=4K`, `sync=standard`, compression/atime off다.
- 두 엔진 모두 같은 clean source SHA `95954ecaaae94757cc3705bc2aacf8c6bb78915f`와 동일 Release 바이너리 SHA-256 `89c6e622b3f05c597a740353b30464bbc30cbe3e910195bfc545967a9dca4c87`을 사용했다. 지정 SHA의 OCI checkout에 있던 검증된 native ARM64 빌드를 재사용했다. 빌드는 `cargo build --release --target aarch64-unknown-linux-gnu -p dodb-storage --bin phase0-bench`, Cargo 설정은 `-C target-feature=+crc`였다. 빌드 출처, Cargo/Rust 버전, checkout 및 바이너리 지문은 `build-provenance.json`, `build-candidate.log`, `binary-provenance.json`, `preflight.txt`, `postflight.txt`에 있다.
- `main-btree`와 `planned-blink`, `parallel_workers=0`을 비교했다. `parallel-blink`로 전환하지 않았다. borrowed-page view는 비활성, 읽기 관측 지표는 활성이다. parallel worker dispatch는 모두 0이었다.
- 공통 입력은 warmup 1초, 요청 측정 120초, repetition 1, seed `0xd0db2026`, Tokio worker 2, working set 4096, cache 256, key/value 16/64 bytes, uniform, real sync다. 성공 반환은 `ProductionFile`의 `sync_data` 완료를 따른다. Query16은 기존 분산 generator, limit 16과 실제 요청 fingerprint를 사용했다.
- 혼합은 writer 1개와 reader 1개로 총 동시 client 2개다. 읽기 요청 1건과 쓰기 transaction 1건을 기준으로 50/50을 적용했다. 폭은 쓰기 transaction당 mutation 수다. 혼합 verifier의 event 및 memory 한도는 변경하지 않았다.
- `--window-seconds`는 사용하지 않았다. 처리량은 요청 성공 수를 JSONL의 공통 측정 시작부터 마지막 client 완료까지의 `client_latest_completion_offset_ns`로 나눠 계산했다. client drain offset은 120,000.000326–120,001.663276ms였고, JSONL의 RSS sampler 종료를 포함한 `duration_ms`는 120,000–120,002ms다. 외부 프로세스 경과는 DB 준비·종료를 포함해 121.549–121.739초다. 구간별 처리량은 수집하지 않았으며 추정하지 않는다.
- 지연 p50/p95/p99는 bounded reservoir 표본 기반이다. Get·Query read와 혼합 read/write 각각의 표본 수는 16,384개다. 요청 전체의 정확한 지연 분포로 해석하지 않는다. RSS는 기존 측정 구간 sampler 결과다.

실제 조건과 명령은 `conditions.tsv`, `commands.txt`, `manifest.tsv`에 기록했다. 원본 JSONL과 stdout/stderr 로그는 각각 `raw/`, `logs/`에 있다. 체크섬은 `SHA256SUMS`에 있다.

## 실행별 성능·자원

처리량의 분모는 drain 포함 측정 구간이다. 혼합 지연의 왼쪽은 read, 오른쪽은 write transaction이다. 지연 단위는 µs이며 각 지연 수치는 해당 분류에서 표본 16,384개를 사용했다.

| workload | engine | aggregate ops/s | read/s | Query/s | write txn/s | mutation/s | read p50/p95/p99 | write p50/p95/p99 | RSS peak MiB | CPU machine % |
|---|---|---:|---:|---:|---:|---:|---|---|---:|---:|
| Get | main-btree | 1,252,767 | 1,252,767 | — | — | — | 0.60/0.68/0.72 | — | 23.97 | 49.98 |
| Get | planned-blink | 1,228,567 | 1,228,567 | — | — | — | 0.60/0.68/0.76 | — | 27.25 | 49.95 |
| Query16 | main-btree | 211,834 | 211,834 | 211,834 | — | — | 3.92/5.04/5.32 | — | 25.20 | 50.04 |
| Query16 | planned-blink | 191,147 | 191,147 | 191,147 | — | — | 4.48/5.56/5.72 | — | 27.41 | 49.99 |
| mixed 폭 4 | main-btree | 1,461.3 | 730.6 | — | 730.6 | 2,922.5 | 2.00/3.96/5.76 | 1,221/2,032/3,334 | 34.51 | 57.24 |
| mixed 폭 4 | planned-blink | 2,241.3 | 1,120.7 | — | 1,120.7 | 4,482.7 | 3.12/4.64/5.48 | 835/1,061/1,351 | 32.38 | 10.68 |
| mixed 폭 8 | planned-blink | 2,085.1 | 1,042.6 | — | 1,042.6 | 8,340.6 | 3.16/4.68/5.48 | 883/1,133/2,127 | 35.16 | 13.44 |
| mixed 폭 8 | main-btree | 785.7 | 392.9 | — | 392.9 | 3,142.9 | 3.20/4.64/5.68 | 2,539/2,931/4,075 | 34.80 | 56.02 |

mixed read/write 성공 건수는 각각 폭 4 main `87,677/87,677`, planned `134,481/134,482`, 폭 8 planned `125,109/125,110`, main `47,145/47,145`다. client별 attempted/successful 합계는 전체와 일치했다. client 완료 편차는 네 mixed 실행에서 약 1.23ms, 12.52µs, 18.20µs, 1.66ms였다.

## 정합성·안전장치

- 8개 채택 실행 모두 JSONL 1개, source SHA 일치, `errors/overloads/conflicts=0`, client aggregation 통과, borrowed-page 비활성, 읽기 관측 지표 활성, `parallel_workers=0`을 기록했다. planned-blink는 실행 종료 시 active generation pin이 0이었다.
- Get 표본 value/revision 검증은 두 실행에서 통과했다. 검사 표본은 main 1,541,528건, planned 1,511,949건이며 실패·indeterminate·누락은 0이다.
- Query 응답은 성공한 요청마다 key, 정렬 순서, 16행 수, value를 검사했다. main 25,420,125요청/406,722,000행, planned 22,937,667요청/367,002,672행이며 실패 0이다. Query 응답 검증은 revision을 검사하지 않는다. 두 실제 입력의 동일 요청 수 fingerprint checkpoint 175개가 모두 일치했고, 공통 마지막 checkpoint는 22,937,600요청이다.
- 혼합 revision-history 검증은 네 실행 모두 `passed`다. 이력 event 수는 4,411–11,525로 65,536 한도 이내, 추정 peak memory는 3,900,832–6,786,048 bytes로 8,388,608 bytes 한도 이내였다. 누락 event, indeterminate read, 실패, ambiguous write는 모두 0이다.
- 측정 구간 RSS는 8/8 `complete`, 표본 11,784–11,923개, 수집 실패 0, peak 양수다. 전체 실행의 `client_completion_skew_ns`를 기록했다.
- 실제 drain 포함 구간, client 합계, 검증 세부사항, RSS와 CPU는 `per-run.tsv`; 자동 대조는 `validation.json`에 있다.

## 보존한 무효 시도와 해석 범위

측정 중 세 가지 실행 관리 오류를 발견했다. 성공 표본에 섞지 않았으며 원본은 `invalid/`와 `attempt-ledger.tsv`에 보존했다.

1. 첫 Get의 JSONL은 상대 출력 경로 때문에 source checkout 안에 만들어졌다. 레코드 SHA-256을 보존해 결과 `raw/`로 복사하고 생성된 경로만 제거했다. 측정 레코드는 요청 조건·source SHA와 검증을 통과했고, checkout은 이후 clean임을 확인했다.
2. 두 번째 planned-blink Get 시도는 잘못 조립된 출력 경로를 발견해 105초 후 중단했다. JSONL 레코드가 없어 유효 측정으로 세지 않았다. 그 다음 시도부터 `--output` 전체를 OCI 결과 경로로 고정했다.
3. 처음 수행한 네 mixed 실행은 `--mixed-clients`가 writer 1 + reader 1 대신 mixed client 하나를 구성했다. JSONL과 로그를 보존하고 무효로 제외했다. 옵션을 제거해 writer/reader 역할의 client 두 개가 생성되는지 확인한 뒤 네 실행을 재측정했고, 그 결과만 채택했다.

이 보고서는 같은 source SHA의 main-btree와 planned-blink를 조건별 120초 동안 비교한 단일 실행 기록이다. 과거 `oci-a1-query-cursor-95954ecaa`의 5초 B/C는 `parallel-blink`, `blink_workers=2` 설정이며 이 결과의 `planned-blink`, `parallel_workers=0`과 별도 구성이다. 앞선 5초 parallel-blink 결과를 이번 지속 측정의 반복이나 동일 엔진 설정으로 취급하지 않는다. real sync 지속 실행 및 과거 복구 테스트는 crash 장애 주입 증거가 아니다.
