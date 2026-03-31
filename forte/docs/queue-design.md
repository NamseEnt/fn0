# Queue Task System Design

## Overview

Forte app backend에서 deterministic하게 작업을 처리하기 위한 queue 시스템.
Per-site 큐로 동작하며, 저장소는 Turso DB를 사용한다.

## User API

### Queue Task 정의

```
rs/src/queue_task/{task_name}.rs
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Input {
    pub user_id: String,
    pub level: u8,
}

pub async fn handle(input: Input) -> anyhow::Result<()> {
    // task processing logic
}
```

- `Input`: Serialize + Deserialize 필수
- `Output` 없음
- `ForteRequest` 없음 (HTTP context 불필요)
- `handle`은 async fn

### Enqueue

backend code 어디서든:

```rust
crate::enqueue::my_task(MyTaskInput { user_id, level }).await?;
```

`generate_routes.rs`가 `enqueue` 모듈을 생성한다.
각 queue_task 파일마다 enqueue 함수가 하나씩 생성된다.

## DB Schema

### Queue Table: `__forte_queue`

| column | type | description |
|--------|------|-------------|
| id | TEXT (UUID v7) | PK, 시간순 정렬 가능 |
| task_name | TEXT | queue_task 이름 |
| payload | TEXT (JSON) | Input을 직렬화한 값 |
| status | TEXT | `pending`, `processing` |
| retry_count | INTEGER | 현재까지 시도 횟수 (0부터 시작) |
| max_retries | INTEGER | 최대 재시도 횟수 (기본 3) |
| created_at | TEXT (ISO 8601) | 생성 시각 |
| updated_at | TEXT (ISO 8601) | 마지막 상태 변경 시각 |

### Dead Queue Table: `__forte_dead_queue`

| column | type | description |
|--------|------|-------------|
| id | TEXT (UUID v7) | PK, 원본 queue id와 동일 |
| task_name | TEXT | queue_task 이름 |
| payload | TEXT (JSON) | 원본 Input |
| error_message | TEXT | 마지막 실패 에러 메시지 |
| retry_count | INTEGER | 총 시도 횟수 |
| created_at | TEXT (ISO 8601) | 원본 생성 시각 |
| died_at | TEXT (ISO 8601) | dead queue로 이동된 시각 |

## Processing Flow

### Enqueue (generated code)

1. `enqueue::my_task(input)` 호출
2. Input을 JSON serialize
3. `__forte_queue`에 INSERT (status=`pending`, retry_count=0)

### Poll & Execute

polling 주체 (dev server / fn0-worker)가 주기적으로:

1. `/__forte_queue_task/poll` 내부 엔드포인트를 호출
2. WASM handler가 `__forte_queue`에서 `pending` 태스크들을 SELECT
3. 각 태스크의 status를 `processing`으로 UPDATE
4. 병렬로 `handle(input)` 실행
5. 성공 시: 해당 row DELETE
6. 실패 시:
   - retry_count + 1 < max_retries → status를 `pending`으로 복귀, retry_count 증가
   - retry_count + 1 >= max_retries → `__forte_dead_queue`에 INSERT 후 `__forte_queue`에서 DELETE

### Security

- 외부 HTTP 요청으로 `/__forte_queue_task/*` 경로 접근 시 호스트 레벨에서 차단
- 로컬: dev server가 직접 `fn0.run()` 호출 → 외부 HTTP 경로 필터링
- 프로덕션: fn0-worker가 외부 HTTP에서 해당 경로 요청 시 거부

## Dead Queue Management

### Grafana Metrics

fn0-worker가 주기적으로 dead queue 크기를 측정하여 metrics로 노출한다.

- metric name: `forte_dead_queue_size`
- labels: `task_name`, `code_id`(site)
- Grafana dashboard에 패널 추가, 0 초과 시 alert 설정

### Flush

dead queue의 태스크를 원래 queue로 다시 넣는 기능.

- `/__forte_queue_task/flush` 내부 엔드포인트 (보안 체크 동일)
- 또는 CLI 명령: `forte queue flush [--task-name my_task]`
- 동작: `__forte_dead_queue`에서 SELECT → `__forte_queue`에 INSERT (status=`pending`, retry_count=0) → `__forte_dead_queue`에서 DELETE
- task_name 필터 지원 (특정 태스크만 flush)

## Code Generation

`generate_routes.rs`에서 추가로 생성할 것:

### 1. enqueue 모듈

```rust
// route_generated.rs 내부 (또는 별도 enqueue_generated.rs)
pub mod enqueue {
    pub async fn my_task(input: crate::queue_task::my_task::Input) -> anyhow::Result<()> {
        let payload = forte_sdk::serde_json::to_string(&input)?;
        let id = forte_sdk::Uuid::now_v7().to_string();
        let now = forte_sdk::now().to_rfc3339();
        // __forte_queue에 INSERT
        forte_sdk::forte_db::turso()
            .execute(
                "INSERT INTO __forte_queue (id, task_name, payload, status, retry_count, max_retries, created_at, updated_at) VALUES (?, ?, ?, 'pending', 0, 3, ?, ?)",
                &[&id, "my_task", &payload, &now, &now],
            )
            .await?;
        Ok(())
    }
}
```

### 2. poll handler

`/__forte_queue_task/poll` 엔드포인트:

1. `__forte_queue`에서 pending 태스크 N개 SELECT
2. status를 processing으로 UPDATE
3. task_name에 따라 match하여 적절한 handler 호출
4. 결과에 따라 DELETE 또는 dead queue 이동

```rust
match task_name.as_str() {
    "my_task" => {
        let input: crate::queue_task::my_task::Input = serde_json::from_str(&payload)?;
        crate::queue_task::my_task::handle(input).await
    }
    // ... other tasks
}
```

### 3. flush handler

`/__forte_queue_task/flush` 엔드포인트:

- dead queue → queue 이동 로직

## Polling Configuration

### Local (dev server)

- `forte dev` 실행 시 백그라운드 tokio task로 polling loop 시작
- 간격: 1초 (설정 가능)
- `fn0.run("backend", "", poll_request, None)` 호출

### Production (fn0-worker)

- fn0-worker에 polling loop 추가
- 간격: 1초 (설정 가능)
- 등록된 code(site) 각각에 대해 poll 요청 실행

## Table Initialization

- `/__forte_queue_task/init` 내부 엔드포인트 또는
- enqueue/poll 시 테이블이 없으면 자동 CREATE TABLE
- CREATE TABLE IF NOT EXISTS 사용

## Implementation Order

1. DB schema (queue table, dead queue table, CREATE TABLE IF NOT EXISTS)
2. Code generation: queue_task 스캔, enqueue 모듈 생성, poll/flush handler 생성
3. Security: 호스트 레벨 경로 차단 (dev server, fn0-worker)
4. Polling loop: dev server, fn0-worker에 추가
5. Dead queue flush: 엔드포인트 또는 CLI
6. Grafana metrics: dead queue size 노출 및 dashboard/alert 설정
