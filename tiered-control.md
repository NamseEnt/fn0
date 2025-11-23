네, 개발 시 참고하실 수 있도록 **Rust/Tokio 기반의 Tiered FaaS 아키텍처** 구현 가이드를 정리해 드립니다.

---

# 🛠️ 개발 메모: Rust/Tokio 기반 Tiered FaaS 아키텍처

## 1. 시스템 개요 (Architecture Overview)

*   **목표:** 외부 DB(Redis) 없이 인스턴스 간 협업으로 웜업 정보를 공유하고 효율적으로 라우팅하는 FaaS.
*   **구조:** **Tiered Architecture (Master - Replica - Worker)**
    *   **Master (1 Node):** 쓰기(Write) 권한 독점, 레플리카 관리, 배포 오케스트레이션.
    *   **Replica ($\approx \sqrt{N}$ Nodes):** 마스터로부터 데이터를 받아 캐싱, 일반 Worker들의 읽기(Read) 요청 처리.
    *   **Worker:** 실제 함수 실행 담당. 웜업 시 마스터에 등록, 실행 시 레플리카에 조회.
*   **네트워크:**
    *   **Gossip (UDP):** 노드 생사 확인(Liveness), 클러스터 멤버십, 마스터 선출 정보.
    *   **gRPC/TCP:** 웜업 메타데이터 동기화, 마스터 승계(Handover).

## 2. 모듈별 구현 상세

### A. 동시성 모델 (Concurrency & Actor)
**핵심:** `Mutex` 지옥을 피하고, FaaS 실행 로직에 의해 클러스터 관리가 굶어 죽지 않도록 **Actor Pattern** 사용.

*   **ClusterManager (Actor):**
    *   전체 클러스터 상태(`ClusterState`)를 유일하게 소유(Ownership).
    *   `tokio::mpsc::channel`을 통해서만 메시지를 처리 (Message Driven).
    *   이 Actor는 **별도의 Tokio Task** (혹은 필요시 `std::thread`와 채널 연동)로 실행되어 항상 높은 반응성을 유지.
*   **ClusterState (Data):**
    *   `routing_table: HashMap<FuncID, Vec<NodeIP>>`
    *   `topology: Vec<NodeInfo>` (Uptime 기준 정렬된 멤버 리스트)

```rust
// 메시지 정의
enum ClusterMsg {
    Heartbeat(NodeId),
    RegisterFunction { func_id: String, ip: String }, // Warm-up 알림
    QueryFunction { func_id: String, resp: oneshot::Sender<Option<String>> }, // 조회
    DeploymentHandover(StateSnapshot), // 배포 시 상태 이관
}

// 액터 루프 (단일 소유권으로 락 불필요)
async fn cluster_manager_loop(mut rx: mpsc::Receiver<ClusterMsg>, mut state: ClusterState) {
    while let Some(msg) = rx.recv().await {
        match msg {
            // 메시지 처리 로직
        }
    }
}
```

### B. 토폴로지 및 역할 결정 (Topology)
*   **Discovery:** `memberlist`와 유사한 가십 프로토콜 사용.
*   **Role Determination (Dynamic):**
    *   모든 노드는 전체 노드 리스트를 `Uptime`(시작 시각) 순으로 정렬.
    *   `Rank 0`: **Master** (자신이 0번이면 마스터 로직 가동).
    *   `Rank 1 ~ k` ($k \approx \sqrt{N}$): **Replica** (마스터 구독).
    *   나머지: **Worker**.
*   **Failover:** 마스터(Rank 0)가 가십에서 사라지면, Rank 1이 즉시 Rank 0가 되어 마스터 권한 승계.

### C. 라우팅 로직 (Request Handling)
외부 로드밸런서에 의해 라운드로빈으로 요청이 들어왔을 때의 흐름.

1.  **Local Cache Check:** 내 메모리(LRU Cache)에 해당 함수 실행 가능한 노드 정보가 있는지 확인.
2.  **Tiered Lookup (If Cache Miss):**
    *   자신이 할당된 **Replica**에게 `QueryFunction` RPC 호출.
    *   (참고: 내가 Master나 Replica라면 내 메모리 바로 조회).
3.  **Fallback & Execution:**
    *   정보가 없거나(None), 조회 실패 시 → **"내가 직접 처리(Self Execution)"**.
    *   처리 시작과 동시에 `RegisterFunction` 메시지를 **Master**에게 발송 (비동기).
    *   Master는 이를 받아 Replica들에게 전파.

---

## 3. 배포 및 마스터 승계 (Deployment & Handover)

**전략:** 배포 주체는 CI/CD 툴이 아닌 **현재의 Master**. (Active Handover)

### Step 1. 감지 및 준비
*   Master는 주기적으로 이미지 레지스트리/설정을 폴링.
*   새 버전(v2)이 있으면 인프라 API를 호출해 **v2 인스턴스(Canary)**를 소수 기동.

### Step 2. 데이터 이관 (Handover)
*   v2 인스턴스가 가십에 합류.
*   Master(v1)는 v2 중 하나를 **Target Master**로 선정.
*   Master(v1)는 자신의 `routing_table` 전체를 직렬화하여 Target Master(v2)에게 RPC 전송 (`HandoverState`).
*   Target Master(v2)는 이를 받아 메모리에 로드하고 "준비 완료" 응답.

### Step 3. 트래픽 전환 및 종료
*   Master(v1)는 들어오는 모든 **쓰기 요청(Register)**을 v2로 리다이렉트.
*   Master(v1)는 인프라 API를 통해 v1 노드들을 점진적 종료(Drain) 및 v2 노드 스케일 아웃.
*   마지막으로 Master(v1) 스스로 종료.

---

## 4. 데이터 흐름 요약

### Write Path (웜업 발생 시)
> Worker(나) -> Master -> (Batching) -> Replicas

1.  Worker가 함수 실행 및 웜업 완료.
2.  Worker는 Master에게 `Register(func_id, my_ip)` 전송.
3.  Master는 메모리 업데이트 후, 약간의 버퍼링(예: 100ms) 후 변경분을 Replica들에게 브로드캐스트.

### Read Path (요청 처리 시)
> Worker(나) -> Local Cache -> Replica -> Target IP

1.  요청 수신.
2.  로컬에 정보 없으면 Replica에게 질의.
3.  Replica가 IP 목록 반환.
4.  해당 IP로 요청 포워딩.

---

## 5. 핵심 주의사항 (Checklist)

1.  **Starvation 방지:**
    *   FaaS 사용자 함수 실행은 반드시 `tokio::task::spawn_blocking` 등을 이용하거나 별도의 스레드 풀에서 돌려야 함.
    *   `ClusterManager`가 실행되는 태스크가 CPU를 뺏기지 않도록 주의.
2.  **UDP 패킷 사이즈:**
    *   가십으로는 "노드 상태(Alive/Dead)"만 주고받을 것.
    *   "함수 맵 데이터"는 반드시 TCP/gRPC로 처리할 것.
3.  **순환 참조 방지:**
    *   라우팅 시 A -> B -> A로 핑퐁치지 않도록 `Hop Count` 헤더를 두거나, 재전송 횟수 제한(TTL) 구현 필수.
4.  **Graceful Shutdown:**
    *   인스턴스가 `SIGTERM`을 받으면, 즉시 가십으로 "나 나간다(Leave)" 메시지를 뿌려야 다른 노드들이 헛된 요청을 보내지 않음.

이 가이드를 바탕으로 구현하시면, **인프라 비용 0원**에 **배포 무중단성**까지 갖춘 고성능 FaaS 엔진이 될 것입니다.