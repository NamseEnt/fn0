# B-link Query cursor 필터 비용과 OCI 재측정

## 결론

무-overlay Query에서 B-link는 cursor 이전 entry도 `DocumentKey::decode()`한 다음 cursor 비교를 수행하고, 비교마다 cursor를 복제했다. `published_range_state()`는 overlay가 비어 있으면 `query_state()`로 직접 위임한다. overlay가 있으면 별도 base/overlay 병합 경로를 사용한다.

encoded cursor 사전 필터와 borrowed cursor 비교 한 가지만 적용한 뒤 Query16 처리량은 기준 B보다 31.1–32.4% 올랐다. main A 대비 중앙값은 0.911x로 격차가 남았다. Get은 거의 같고 main 대비 중앙값이 약 0.6% 낮았다.

## 변경 및 비용 근거

- `DocumentKey`의 escaped component 인코딩은 bytewise 순서와 구조체 순서가 같다. `dodb-core`의 `encoding_preserves_document_order` property test와 canonical validator 테스트가 통과했다.
- cursor 이하 entry는 먼저 `DocumentKey::validate_encoded()`로 검증한 뒤 건너뛴다. 기존 malformed-key 검사는 유지하면서 decode의 component 할당을 생략한다. 이후 entry는 기존처럼 decode한다.
- PK 경계와 exclusive-after 의미, 정렬, limit, tombstone/overflow, leaf 연결·cycle 검사, GenerationPin/page reuse, read metrics, overlay 병합 코드는 유지했다.
- Query16 입력은 PK마다 32행 중 16개를 읽고 시작 위치 0–16을 uniform 선택한다. 같은 seed에서 측정 Query 입력 지문은 A/B/C 간 반복별 공통 15 checkpoint가 모두 일치했다. 결정적 generator를 replay한 same-PK 이전 행 수는 B에서 반복당 5.76–5.81M개, C에서 7.62–7.67M개이며 요청당 평균 약 8개다.
- 결정적 입력 생성기를 재생해 계산한 하한에서 B의 cursor 이전 same-PK entry decode/clone은 반복당 5.76–5.81M회, C의 성공 처리량에 해당하는 trace 길이에서는 7.62–7.67M회가 생략될 수 있었다. 이 횟수는 입력 기반 추정이며 실제 CPU 시간이나 allocation을 직접 측정한 값이 아니다. cursor가 있는 Query는 첫 cursor 초과 행에서 첫 비교와 본 비교로 clone을 2회 하고, 뒤따르는 반환 행에서 각 1회씩 더 했다. 이를 포함한 전체 제거 clone 하한은 B 17.28–17.43M, C 22.85–23.01M회다. leaf 시작에 인접한 앞선 PK entry가 있으면 실제 제거 수는 이보다 많다.
- 이 비용 재생산은 성공한 측정 Query 요청만 사용한다. 준비·seeding·warmup 비용은 포함하지 않았고, 진단용 계측은 정식 성능 바이너리에서 활성화하지 않았다.

정확한 회차별 수치는 `results/query-cursor-cost-diagnostic.json`에 있다.

## 구성 및 조건

OCI A1 `opc@217.142.246.204`, ARM64 native Release, real sync, reader 1, Tokio worker 2, working set 4096, cache 256, key/value 16/64 bytes, uniform, Query limit 16, warmup 1초, 측정 5초를 사용했다. Seed는 `0xd0db2026`–`0xd0db2028`; borrowed-page는 세 구성 모두 기본 비활성이다.

| 구성 | 소스 SHA | 실행 조건 | 바이너리 SHA-256 |
|---|---|---|---|
| A main-btree | `4bc4d42e4d816f4a11428f5423d89a4c35494f9b` | `main-btree` | `1de9bab43902c81770afa6fa466ecc19e4307d7c71444eb05232e51df17553c3` |
| B 기준 | `4bc4d42e4d816f4a11428f5423d89a4c35494f9b` | `parallel-blink`, `blink_workers=2` | `1de9bab43902c81770afa6fa466ecc19e4307d7c71444eb05232e51df17553c3` |
| C 후보 | `95954ecaaae94757cc3705bc2aacf8c6bb78915f` | `parallel-blink`, `blink_workers=2` | `89c6e622b3f05c597a740353b30464bbc30cbe3e910195bfc545967a9dca4c87` |

원격 checkout은 각 지정 SHA의 clean detached checkout이었다. Cargo.lock, Cargo 설정, `phase0-bench.rs` 해시는 동일하고 `dodb` 소스 diff는 `dodb/crates/dodb-storage/src/blink/mod.rs` 하나다. Cargo 설정은 `-C target-feature=+crc`; 각 소스는 ZFS의 별도 target 경로에서 빌드했다. DB와 빌드 임시 경로도 확인된 `/bench/zfs/db` ZFS(`sync=standard`, recordsize 4K)에 있었다. OCI에서 소스를 편집하지 않았다.

B와 C는 모두 `parallel-blink` 엔진에 `blink_workers=2`를 설정했다. 관측된 parallel worker dispatch가 0회였더라도 worker pool 설정 자체가 앞선 `planned-blink` 기준 구성과 다르므로, B/C 비교는 같은 parallel-blink 설정 안에서의 비교로 해석한다. 실행 순서는 반복 1 `A→B→C`, 반복 2 `C→A→B`, 반복 3 `B→C→A`였다. 성공 수를 측정 RSS 창의 실제 경과시간으로 나눠 처리량을 재계산했다. seeding과 warmup은 처리량 시간에 포함하지 않았다.

## 반복별 처리량 비율

| workload | 반복 | seed | C/B | B/A | C/A |
|---|---:|---|---:|---:|---:|
| Get | 1 | `0xd0db2026` | 1.0090x | 0.9886x | 0.9975x |
| Get | 2 | `0xd0db2027` | 0.9981x | 0.9924x | 0.9905x |
| Get | 3 | `0xd0db2028` | 1.0077x | 0.9866x | 0.9942x |
| Query16 | 1 | `0xd0db2026` | 1.3110x | 0.6915x | 0.9066x |
| Query16 | 2 | `0xd0db2027` | 1.3219x | 0.7393x | 0.9773x |
| Query16 | 3 | `0xd0db2028` | 1.3243x | 0.6876x | 0.9106x |

## 처리량, 지연, RSS, CPU

중앙값은 세 실행의 중앙값이며 범위는 최솟값–최댓값이다. 지연은 실행별 read latency의 p50/p95/p99다. RSS는 측정 창 내부의 관측 peak다.

| workload | 구성 | 처리량 ops/s 중앙값 (범위) | p50 us | p95 us | p99 us | CPU 한 코어 환산 | RSS peak MiB 중앙값 (범위) |
|---|---|---:|---:|---:|---:|---:|---:|
| Get | A | 1,240,563 (1,236,130–1,243,013) | 0.600 | 0.680 | 0.760 | 99.788% | 25.551 (25.324–25.621) |
| Get | B | 1,223,902 (1,222,045–1,233,591) | 0.600 | 0.680 | 0.760 | 99.982% | 27.531 (27.469–27.629) |
| Get | C | 1,233,083 (1,231,202–1,233,353) | 0.600 | 0.680 | 0.760 | 99.789% | 27.352 (27.168–27.453) |
| Query16 | A | 209,305 (196,102–209,998) | 3.960 | 5.120 | 5.360 | 99.789% | 26.715 (26.645–26.734) |
| Query16 | B | 144,977 (143,912–145,211) | 6.240 | 7.801 | 8.040 | 99.387% | 27.594 (27.562–27.598) |
| Query16 | C | 190,585 (190,375–191,646) | 4.480 | 5.560 | 5.720 | 100.182% | 27.473 (27.336–27.520) |

CPU 한 코어 환산은 모두 약 100%였다. C의 Query 지연은 B보다 낮아졌지만 main보다 높다. C의 Query RSS는 A보다 약 0.76 MiB 높고 B와 거의 같았다. Get RSS는 Blink 두 구성에서 A보다 약 1.8 MiB 높았다.

## 정합성 및 계측

- `dodb-storage` 전체: 181 passed, 0 failed, 4 ignored. `phase0-bench`: 42 passed. 복구 통합: 8 passed. 추가 cursor 회귀 테스트도 별도 실행해 통과했다.
- `dodb-core`: 6 passed. ordering, canonical validation, arbitrary binary round-trip 테스트를 포함한다.
- 회귀 테스트는 cursor 없음, limit 0, 첫 행 전, 기존 cursor, 삭제된 cursor, 마지막 행 뒤, 여러 leaf, PK 경계, overflow value를 확인한다.
- OCI 18/18 실행 성공. Get 표본은 value/revision을 검증했다. Query 응답은 key, 순서, 행 수, value를 검사했으며 revision 검사를 수행했다고 볼 근거는 없다. 입력 지문과 read metrics도 통과했다. Blink active generation pin은 모든 실행 종료 시 0이었다.
- 측정 창 RSS 내부 계측은 18/18 complete, 표본 485–497개였다. 보조 `/proc` RSS sampler는 종료 경쟁으로 총 20회 실패(16개 실행에서 각 1회, 2개 실행에서 각 2회)를 기록했다. 두 계측은 별도로 보존했다.
- 무효 성능 실행과 재실행은 없다. 실행 전 환경 조회 명령의 ZFS 속성 인자 오류가 있었으나 벤치마크 프로세스 시작 전이었다. 인자를 바로잡아 ZFS를 확인한 후 실행했다.

이번 한 변경은 Query16 격차를 줄였지만 없애지 못했다. Query C/A 중앙값은 0.911x이며 반복 2는 0.977x, 반복 1·3은 약 0.907x·0.911x였다. Get은 C/A 중앙값 0.994x로 사실상 같지만 약 0.6% 낮다. 추가 최적화나 엔진 전체 채택 판단은 하지 않는다.
