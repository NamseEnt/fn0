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

### Enqueue (WASM)

1. `enqueue::my_task(input)` 호출 (WASM 내부, generated code)
2. Input을 JSON serialize
3. `__forte_queue`에 INSERT (status=`pending`, retry_count=0)

### Poll, Execute, Cleanup (호스트)

호스트(dev server / fn0-worker)가 Turso DB에 직접 접근하여 큐를 관리한다.
호스트는 TURSO_URL과 TURSO_AUTH_TOKEN을 이미 보유하고 있다.

**Poll (claim):**

호스트가 Turso에 직접 쿼리:

```sql
WITH target AS (
  SELECT id FROM __forte_queue
  WHERE status='pending'
     OR (status='processing' AND updated_at < datetime('now', '-60 seconds'))
  LIMIT 1
)
UPDATE __forte_queue SET status='processing', updated_at=?
WHERE id IN (SELECT id FROM target) RETURNING *
```

- pending 태스크와 timeout된 processing 태스크를 하나의 쿼리로 처리
- UPDATE ... RETURNING으로 atomic claim
- 다중 워커 환경에서 경합 안전

**Execute:**

호스트가 claim된 태스크의 task_name + payload를 WASM에 전달:

- `/__forte_queue_task/execute` 내부 엔드포인트 호출
- 태스크 1개당 별도 WASM 호출 (CPU 타임 제한 준수)
- WASM은 task_name에 따라 handler를 match하여 실행

```rust
match task_name.as_str() {
    "my_task" => {
        let input: crate::queue_task::my_task::Input = serde_json::from_str(&payload)?;
        crate::queue_task::my_task::handle(input).await
    }
    // ... other tasks
}
```

**Cleanup:**

호스트가 WASM 실행 결과에 따라 Turso에 직접 처리:

- 성공: `__forte_queue`에서 해당 row DELETE
- 실패 (retry_count + 1 < max_retries): status를 `pending`으로 복귀, retry_count 증가
- 실패 (retry_count + 1 >= max_retries): `__forte_dead_queue`에 INSERT 후 `__forte_queue`에서 DELETE

### Security

- 외부 HTTP 요청으로 `/__forte_queue_task/*` 경로 접근 시 호스트 레벨에서 차단
- 로컬: dev server가 외부 HTTP에서 해당 경로 요청 시 거부
- 프로덕션: fn0-worker가 외부 HTTP에서 해당 경로 요청 시 거부

## Dead Queue Management

### Grafana Metrics

호스트(fn0-worker)가 주기적으로 Turso에 직접 쿼리하여 dead queue 크기를 측정, OTLP를 통해 Grafana에 push한다.

```sql
SELECT task_name, COUNT(*) as count FROM __forte_dead_queue GROUP BY task_name
```

- metric name: `forte_dead_queue_size`
- labels: `task_name`, `code_id`(site)
- type: gauge
- Grafana dashboard에 패널 추가, 0 초과 시 alert 설정
- 기존 OTLP 파이프라인 활용 (추가 인프라 불필요)

### Flush

dead queue의 태스크를 원래 queue로 다시 넣는 기능.

CLI 명령으로 제공:

```
forte queue flush [--task-name my_task]
```

- 동작: `__forte_dead_queue`에서 SELECT → `__forte_queue`에 INSERT (status=`pending`, retry_count=0) → `__forte_dead_queue`에서 DELETE
- task_name 필터 지원 (특정 태스크만 flush)
- 필터 없이 실행 시 모든 dead 태스크를 flush
- 로컬: local sqld에 직접 접속하여 실행
- 프로덕션: Turso DB에 직접 접속하여 실행 (TURSO_URL, TURSO_AUTH_TOKEN 사용)

## Code Generation

`generate_routes.rs`에서 추가로 생성할 것:

### 1. enqueue 모듈

```rust
pub mod enqueue {
    pub async fn my_task(input: crate::queue_task::my_task::Input) -> anyhow::Result<()> {
        let payload = forte_sdk::serde_json::to_string(&input)?;
        let id = forte_sdk::Uuid::now_v7().to_string();
        let now = forte_sdk::now().to_rfc3339();
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

### 2. execute handler

`/__forte_queue_task/execute` 엔드포인트:

1. 호스트로부터 task_name + payload를 받음
2. task_name에 따라 match하여 handler 호출
3. 결과(성공/실패)를 응답으로 반환

## Polling Loop

### 호스트의 역할

polling loop는 호스트(dev server / fn0-worker)에서 실행:

1. Turso에 직접 claim 쿼리 (UPDATE ... RETURNING)
2. 태스크가 있으면 `/__forte_queue_task/execute` WASM 호출
3. 결과에 따라 Turso에 직접 cleanup (DELETE / retry / dead queue)
4. 반복

### Local (dev server)

- `forte dev` 실행 시 백그라운드 tokio task로 polling loop 시작
- 간격: 1초
- Turso 접속: `http://127.0.0.1:8080` (local sqld)

### Production (fn0-worker)

- fn0-worker에 polling loop 추가
- 간격: 1초
- 등록된 code(site) 각각에 대해 poll 실행
- 다중 워커가 동시에 poll해도 UPDATE ... RETURNING으로 경합 안전

## Table Initialization

- 호스트가 polling loop 시작 시 CREATE TABLE IF NOT EXISTS 실행
- Turso에 직접 DDL 실행
- 별도 migration 불필요

## Implementation Order

1. DB schema (queue table, dead queue table, CREATE TABLE IF NOT EXISTS)
2. Code generation: queue_task 스캔, enqueue 모듈 생성, execute handler 생성
3. Security: 호스트 레벨 경로 차단 (dev server, fn0-worker)
4. Polling loop + cleanup: dev server, fn0-worker에 추가 (Turso 직접 접근)
5. Dead queue flush: CLI 명령 (`forte queue flush`)
6. Grafana metrics: OTLP를 통한 dead queue size 노출 및 dashboard/alert 설정
