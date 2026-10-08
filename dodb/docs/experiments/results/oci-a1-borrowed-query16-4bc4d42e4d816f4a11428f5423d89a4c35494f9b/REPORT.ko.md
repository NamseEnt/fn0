# borrowed-page 옵션의 Query16 및 Get 비교

- 공통 소스 SHA: `4bc4d42e4d816f4a11428f5423d89a4c35494f9b`
- 구성 A: `main-btree`, 기본 바이너리
- 구성 B: `planned-blink`, 기본 feature 바이너리
- 구성 C: `planned-blink`, `blink-borrowed-page-views` feature 바이너리
- 18회 측정: 구성 3 × Get/Query16 × 반복 3, 전부 5초 측정과 1초 warmup, real sync
- 입력: ARM64 OCI A1, reader 1, Tokio worker 2, working set 4096, cache 256, key/value 16/64 bytes, uniform, Query limit 16
- 처리량은 `successful_reads / process_rss_window_end_offset_ns`로 다시 계산했다. 정수 millisecond 시간은 분모로 사용하지 않았다.
- p50/p95/p99 요약은 실행별 percentile의 반복 중앙값과 범위다. 요청 표본을 합쳐 percentile을 다시 계산하지 않았다.

## 짝 비교 비율

| workload | 반복 | seed | C/B borrowed 효과 | B/A main 대비 | C/A main 대비 |
|---|---:|---:|---:|---:|---:|
| get | 1 | `0xd0db2026` | 1.0164x | 0.9845x | 1.0006x |
| get | 2 | `0xd0db2027` | 1.0142x | 0.9856x | 0.9996x |
| get | 3 | `0xd0db2028` | 1.0198x | 0.9818x | 1.0013x |
| query | 1 | `0xd0db2026` | 1.0058x | 0.6927x | 0.6968x |
| query | 2 | `0xd0db2027` | 1.0061x | 0.6933x | 0.6975x |
| query | 3 | `0xd0db2028` | 1.0025x | 0.6937x | 0.6954x |

## workload별 요약

| workload | 구성 | 처리량 중앙값 ops/s (범위) | 반복 범위/중앙값 | C/B | B/A | C/A | p50 us 중앙값 (범위) | p95 us 중앙값 (범위) | p99 us 중앙값 (범위) | CPU 한 코어 중앙값 | 관측 RSS peak MiB 중앙값 (범위) |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| get | A main-btree | 1254825 (1254559–1259296) | 0.38% | — | — | — | 0.560 (0.560–0.560) | 0.680 (0.680–0.680) | 0.720 (0.720–0.720) | 99.59% | 25.48 (25.12–25.50) |
| get | B planned-blink 기본 | 1236444 (1231958–1239795) | 0.63% | — | 0.9845x | — | 0.600 (0.600–0.600) | 0.680 (0.680–0.680) | 0.720 (0.720–0.760) | 99.79% | 32.05 (32.02–32.07) |
| get | C planned-blink borrowed | 1256400 (1254046–1260102) | 0.48% | 1.0164x | — | 1.0006x | 0.560 (0.560–0.560) | 0.680 (0.680–0.680) | 0.720 (0.720–0.720) | 99.79% | 31.98 (31.66–32.09) |
| query | A main-btree | 210303 (210026–210411) | 0.18% | — | — | — | 3.920 (3.920–3.960) | 5.080 (5.080–5.080) | 5.320 (5.320–5.321) | 99.59% | 26.68 (26.58–26.69) |
| query | B planned-blink 기본 | 145807 (145494–145967) | 0.32% | — | 0.6933x | — | 6.200 (6.200–6.200) | 7.761 (7.760–7.800) | 8.000 (8.000–8.040) | 99.99% | 32.09 (32.03–32.14) |
| query | C planned-blink borrowed | 146341 (146328–146690) | 0.25% | 1.0058x | — | 0.6968x | 6.200 (6.200–6.200) | 7.760 (7.760–7.760) | 8.000 (7.960–8.000) | 99.99% | 31.95 (31.95–32.05) |

## 유효성 및 자원 측정

- 18/18 run이 source SHA, binary SHA, feature 기록, 정확성 검사, 양수 성공 수, 오류/overload/conflict 0을 통과했다. 유효 run 실패는 0건이다.
- B와 C의 종료 시 active generation pin은 모두 0이다. 두 Blink 구성에서 generation pin 수와 읽기 관측 횟수는 성공 read 수와 일치한다.
- Get은 9/9 실행의 seed value/revision 검사가 통과했고 indeterminate가 0이다. Query는 9/9 실행에서 성공 요청 모두 응답 검사를 거쳤고 각 요청당 16행, validation failure 0, client aggregation 통과다.
- Query 입력 fingerprint는 같은 반복의 3개 구성에서 동일 길이 체크포인트 15개(반복당 5개)가 일치했다.
- 측정 구간 내부 RSS 계측은 18/18 complete, 수집 실패 0이다. 표본 수는 496–497, 최대 표본 간격은 10.25–15.18 ms다. 측정 중 최대 RSS는 `process_rss_peak_observed_bytes`를 사용했다.
- 보조 프로세스 RSS 표본은 실행당 320–325개, 최대 간격 20.71–31.54 ms다. 실행 종료 시점 `/proc` 소멸 경합으로 보조 sampler는 실행당 1회 실패를 기록했다. 이는 측정 프로세스 내부 RSS 계측의 0회 실패와 구분해 보존했다.
- CPU 사용률은 한 코어 환산 약 99%였다. 결과는 OCI 2 OCPU 환경에서 단일 reader가 CPU를 대부분 사용한 짧은 측정이다.
- 두 무효 사전 시도는 결과 경로 오기 및 결과/DB namespace 중복으로 벤치마크 시작 전에 종료됐다. 원인은 `logs/preflight-00.txt`, `logs/preflight-01.txt`에 남겼다. 무효 성능 실행은 없다.

## 해석

C/B는 Get에서 반복 중앙값 기준 약 +1.6%, Query16에서 약 +0.6%다. 옵션 효과는 작고, 세 반복 방향은 일관됐다. B/A Query16은 약 0.693x였고 C/A도 약 0.697x라서 borrowed-page 옵션은 재현된 Query16 격차를 사실상 해소하지 못했다. Get은 C/A 약 1.001x로 main과 거의 같았다.

옵션 자체는 소폭의 순수 읽기 개선을 보였지만 Query16 p50/p95/p99는 B와 C가 거의 같고 main보다 높았다. Blink 두 구성은 main보다 측정 RSS가 약 6–7 MiB 높았다. 따라서 이번 실험은 기존 옵션의 제한적인 효과만 보여준다. 단일 OCI host, 3회, 5초 측정이므로 엔진 전체 채택 판단이나 장시간 안정성 판단으로 확장하지 않는다. 120초 지속 측정은 범위 밖이다.

## 빌드와 안전성 테스트

기본 바이너리는 기준 결과의 exact source SHA, phase0-bench source hash, Cargo.lock, Cargo 설정, ARM64 release 출처와 해시가 일치해 재사용했다. feature-on 빌드는 같은 rustc/cargo/toolchain과 `.cargo/config.toml`의 `-C target-feature=+crc` 설정에서 별도 target 디렉터리로 만들었다. 변경한 Cargo feature만 `blink-borrowed-page-views`다.

feature-on OCI storage 테스트는 `dodb-storage` 전체 release 테스트에서 181 passed, 0 failed, 4 ignored, `phase0-bench`에서 42 passed, 0 failed였다. 검사나 안전장치를 비활성화하지 않았다.

## 코드 경로 확인

feature는 `GenerationPin`의 `ReadPageSource::Page`와 `page()`에서 borrowed page view 사용을 선택한다. `GenerationPin` 수명·pin 집계와 Drop 해제는 공통이며, `can_reuse_pages()`는 active pin이 0일 때만 재사용을 허용한다. phase0-bench의 `blink_borrowed_page_views_enabled`, read metric, active pin 필드는 두 바이너리에서 확인했다.
