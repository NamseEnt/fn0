# Phase K Fault and Recovery Status

## Verified

- Existing Phase J synchronous materialization fault and recovery tests pass unchanged.
- A deterministic background page-write failure before checkpoint write leaves the old base authoritative, keeps committed overlays and logical WAL, permits reads, and recovers all acknowledged writes after reopen.
- The worker writes replacement pages and syncs them before it starts writing the alternate checkpoint superblock.
- Runtime WAL reset is restricted to the case where the durable checkpoint watermark covers the current WAL tail. A newer suffix prevents reclaim.

## Required Follow-up

The background worker currently has no test-only fault injection at every requested crash boundary. Add deterministic failures after request, snapshot, build, individual writes, data sync, checkpoint write, checkpoint sync, before/after publication, retirement, and reclaim. Exercise both an uncertain checkpoint write and a crash after durable watermark publication. Verify that every acknowledged transaction is readable after reopen and that no suffix is lost.

The current test proves a pre-checkpoint write error is retryable. Errors after checkpoint writing starts conservatively degrade the current store until reopen because the alternate superblock may be durable. This preserves recoverability but is not yet the requested fully classified retry state machine.
