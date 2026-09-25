# dodb in fn0

fn0 stores control and project documents in dodb. `fn0-control` maps to tenant `u64::MAX`; ordinary fixed-width lowercase base36 project IDs map through `doc_db::dodb_tenant_id`. The special `local` project has no dodb tenant.

## Worker

The production worker uses one shared dodb connection for guest semantic doc-db requests, the manifest poller, the certificate poller, and the WebSocket connection directory. Worker execution contexts do not install the legacy `TursoHijack`; first-party guests that still call a direct Turso or Hrana placeholder fail closed. Guest code using `doc_db::database()` goes through semantic doc-db RPC and is project-isolated by the worker.

Worker and worker-agent instances receive `DODB_ADDR`, `DODB_SERVER_NAME`, and `DODB_ROOT_CERT_PEM_BASE64`. These values are materialized in production Pulumi stack configuration from the current dodb outputs to avoid a Pulumi dependency cycle. Resync all three runtime values whenever dodb is replaced or its certificate rotates. Only the public root certificate is distributed; the dodb private key is not sent to workers.

The worker instance configuration uses one `VM.Standard.A1.Flex` worker with 1 OCPU, 6 GB memory, and a 50 GB boot volume. Collecty stores its queue at `/var/lib/collecty` on the boot filesystem. It has no separate collecty block volume and no automatic CPU autoscaling. Existing live workers are not replaced by source or stack configuration changes alone.

## Control

`fn0Cloud:dbBackend` is the single source of truth for both the control bundle's `FN0_DB_BACKEND` and operator scripts. It accepts only `turso` and `dodb`; missing or unknown values fail closed. Production uses `dodb`, so `bootstrap-fn0-control.sh` and `deploy-fn0-worker.sh` read and write the dodb `fn0-control` tenant and do not use Turso credentials. Switching back to `turso` is not a safe rollback because Turso stopped receiving writes at cutover and is stale.

In dodb mode, raw SQL `doc_query` is explicitly unavailable because dodb does not implement arbitrary SQL. Deploy does not provision a Turso database. Project deletion purges the dodb project tenant through a control-only semantic operation after access and routing cleanup, then removes control identity documents.

`scripts/migrate-project-telemetry-policies.sh` follows `fn0Cloud:dbBackend` for read and check modes. Its historical `--apply` migration fails under dodb before any database or Signy write.

## Operator access

The native `fn0-db-ops` binary is a narrow operator interface fixed to project `fn0-control`. The operator session builds its Linux ARM64 binary, opens OCI Bastion Port Forwarding to dodb SSH, and reuses that SSH tunnel for the rest of the parent script. The binary runs on the private dodb VM and connects to `127.0.0.1:18445` over QUIC with server name `dodb.internal` and `/etc/dodb/server.crt`. The Mac never tunnels QUIC through Bastion TCP.

The binary is cached by SHA-256 under `/home/opc/.cache/fn0/db-ops/` and streamed with gzip over SSH stdin; no SCP or SFTP transfer is used. The cache is retained when the operator session closes. The helper removes the tunnel, Bastion session, and local temporary files when the parent script exits.
