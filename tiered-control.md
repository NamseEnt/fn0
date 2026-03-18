Here is a development guide for a **Rust/Tokio-based Tiered FaaS Architecture** for reference.

---

# Development Memo: Rust/Tokio-based Tiered FaaS Architecture

## 1. Architecture Overview

*   **Goal:** A FaaS that shares warm-up information through inter-instance collaboration and routes efficiently without an external DB (Redis).
*   **Structure:** **Tiered Architecture (Master - Replica - Worker)**
    *   **Master (1 Node):** Exclusive write authority, replica management, deployment orchestration.
    *   **Replica ($\approx \sqrt{N}$ Nodes):** Caches data received from master, handles read requests from regular Workers.
    *   **Worker:** Handles actual function execution. Registers with master on warm-up, queries replicas on execution.
*   **Network:**
    *   **Gossip (UDP):** Node liveness checks, cluster membership, master election information.
    *   **gRPC/TCP:** Warm-up metadata synchronization, master handover.

## 2. Module Implementation Details

### A. Concurrency Model (Concurrency & Actor)
**Key:** Use **Actor Pattern** to avoid `Mutex` hell and prevent cluster management from being starved by FaaS execution logic.

*   **ClusterManager (Actor):**
    *   Sole ownership of the entire cluster state (`ClusterState`).
    *   Processes messages only through `tokio::mpsc::channel` (Message Driven).
    *   This Actor runs as a **separate Tokio Task** (or connected via `std::thread` and channels if needed) to maintain high responsiveness at all times.
*   **ClusterState (Data):**
    *   `routing_table: HashMap<FuncID, Vec<NodeIP>>`
    *   `topology: Vec<NodeInfo>` (Member list sorted by Uptime)

```rust
// Message definitions
enum ClusterMsg {
    Heartbeat(NodeId),
    RegisterFunction { func_id: String, ip: String }, // Warm-up notification
    QueryFunction { func_id: String, resp: oneshot::Sender<Option<String>> }, // Query
    DeploymentHandover(StateSnapshot), // State transfer during deployment
}

// Actor loop (no locks needed due to single ownership)
async fn cluster_manager_loop(mut rx: mpsc::Receiver<ClusterMsg>, mut state: ClusterState) {
    while let Some(msg) = rx.recv().await {
        match msg {
            // Message handling logic
        }
    }
}
```

### B. Topology & Role Determination
*   **Discovery:** Use a gossip protocol similar to `memberlist`.
*   **Role Determination (Dynamic):**
    *   All nodes sort the full node list by `Uptime` (start time).
    *   `Rank 0`: **Master** (activates master logic if self is rank 0).
    *   `Rank 1 ~ k` ($k \approx \sqrt{N}$): **Replica** (subscribes to master).
    *   Rest: **Worker**.
*   **Failover:** If master (Rank 0) disappears from gossip, Rank 1 immediately becomes Rank 0 and assumes master authority.

### C. Routing Logic (Request Handling)
Flow when a request arrives via round-robin from the external load balancer.

1.  **Local Cache Check:** Check if my memory (LRU Cache) has node information capable of executing the function.
2.  **Tiered Lookup (If Cache Miss):**
    *   Make a `QueryFunction` RPC call to the assigned **Replica**.
    *   (Note: If I am a Master or Replica, query my own memory directly).
3.  **Fallback & Execution:**
    *   If no information (None) or query fails → **"Self Execution"**.
    *   Simultaneously send a `RegisterFunction` message to **Master** (async).
    *   Master receives it and propagates to Replicas.

---

## 3. Deployment & Master Handover

**Strategy:** The deployment initiator is not a CI/CD tool but the **current Master**. (Active Handover)

### Step 1. Detection & Preparation
*   Master periodically polls the image registry/config.
*   If a new version (v2) exists, calls the infra API to spin up a small number of **v2 instances (Canary)**.

### Step 2. Data Transfer (Handover)
*   v2 instances join gossip.
*   Master (v1) selects one of v2 as the **Target Master**.
*   Master (v1) serializes its entire `routing_table` and sends it to Target Master (v2) via RPC (`HandoverState`).
*   Target Master (v2) loads it into memory and responds "ready".

### Step 3. Traffic Switch & Shutdown
*   Master (v1) redirects all incoming **write requests (Register)** to v2.
*   Master (v1) gradually drains v1 nodes via infra API and scales out v2 nodes.
*   Finally, Master (v1) shuts itself down.

---

## 4. Data Flow Summary

### Write Path (On Warm-up)
> Worker(self) -> Master -> (Batching) -> Replicas

1.  Worker completes function execution and warm-up.
2.  Worker sends `Register(func_id, my_ip)` to Master.
3.  Master updates memory, then broadcasts changes to Replicas after some buffering (e.g., 100ms).

### Read Path (On Request Handling)
> Worker(self) -> Local Cache -> Replica -> Target IP

1.  Request received.
2.  If no local information, query Replica.
3.  Replica returns IP list.
4.  Forward request to that IP.

---

## 5. Key Considerations (Checklist)

1.  **Starvation Prevention:**
    *   FaaS user function execution must use `tokio::task::spawn_blocking` or run on a separate thread pool.
    *   Ensure the task running `ClusterManager` is not starved of CPU.
2.  **UDP Packet Size:**
    *   Only exchange "node status (Alive/Dead)" via gossip.
    *   "Function map data" must be handled via TCP/gRPC.
3.  **Circular Reference Prevention:**
    *   To prevent A -> B -> A ping-pong during routing, implement a `Hop Count` header or retransmission limit (TTL).
4.  **Graceful Shutdown:**
    *   When an instance receives `SIGTERM`, it must immediately gossip a "Leave" message so other nodes don't send futile requests.

Following this guide will result in a high-performance FaaS engine with **zero infrastructure cost** and **zero-downtime deployments**.
