# Turso to dodb migration

`fn0-db-migrate` inventories, copies, and exactly verifies fn0 document rows from Turso to dodb. Its source is Turso `docs(pk, sk, data, version)`. Migration copies only `pk`, `sk`, and `data`; Turso `version` is intentionally not preserved. dodb creates its own native opaque revision for every destination write.

The active project set is discovered by scanning `fn0-control` rows whose primary key starts with `ProjectDoc/`, decoding each payload, and validating its `project_id` with the existing `doc_db::dodb_tenant_id` mapping. The migration set is always `fn0-control` plus all discovered active projects. `fn0-control` maps to tenant `u64::MAX`; ordinary fixed-width lowercase base36 project IDs use the existing exact mapping. The special `local` project has no dodb tenant and is rejected.

The source reader uses raw ordered SELECT statements and never creates the Turso `docs` table. A missing `docs` table is treated as an empty database. Other SQL errors and malformed rows fail the command. Pages are ordered by `(pk, sk)` and resume with a keyset cursor; OFFSET is not used.

## Commands

The CLI reads `TURSO_GROUP_TOKEN` and `TURSO_DB_HOST_SUFFIX` from its environment. It never accepts the Turso secret as a command-line argument.

```sh
fn0-db-migrate inventory [--page-size 256] [--json]
fn0-db-migrate migrate --apply [--project-id ID ...] [--json]
fn0-db-migrate verify [--project-id ID ...] [--json]
```

`inventory` reads Turso only and reports project IDs, tenant IDs, row counts, and payload byte totals. `migrate` requires `--apply`, copies data, and automatically performs a fresh exact verification. `verify` is read-only. The dodb client uses the existing `DodbConfig::new` / `DodbConnection` API, defaulting to `127.0.0.1:18445`, TLS server name `dodb.internal`, and `/etc/dodb/server.crt`.

Migration is resumable. For every source row it first reads the destination: an equal row is skipped, while a missing or different row is written with `put`. If a put returns an error, the tool immediately reads the key again and accepts the uncertain result only if the destination bytes now exactly equal the source bytes. It never deletes destination rows. Destination-only rows are reported as `extra` and make exact verification fail.

## Cutover readiness

The production worker source uses one shared dodb connection for guest semantic doc-db requests, the manifest poller, the certificate poller, and the WebSocket connection directory. New worker execution contexts do not install the legacy `TursoHijack`; first-party guests that still call a direct Turso or Hrana placeholder will fail closed after replacement. Guest code using `doc_db::database()` continues through semantic doc-db RPC and is project-isolated by the worker.

The control guest selects `FN0_DB_BACKEND=turso` or `FN0_DB_BACKEND=dodb`. In dodb mode, raw SQL `doc_query` is explicitly unavailable because dodb does not implement arbitrary SQL. Deploy does not provision a Turso database. Project deletion purges the dodb project tenant through a control-only semantic operation after access and routing cleanup, then removes control identity documents. Turso remains provisioned and untouched during the confidence window as the rollback/reference source.

`fn0Cloud:dbBackend` is the single source of truth for both the control bundle's `FN0_DB_BACKEND` and operator scripts. It accepts only `turso` and `dodb`; missing or unknown values fail closed. The committed production value remains `turso` before cutover, so landing operator tooling does not change the current authoritative database or control bundle. Change it to `dodb` only during the maintenance cutover. After that switch, `bootstrap-fn0-control.sh` and `deploy-fn0-worker.sh` read and write the dodb `fn0-control` tenant, and they no longer require or use Turso credentials. Switching back to `turso` after dodb has accepted writes is not a safe rollback because Turso is then stale.

`scripts/migrate-project-telemetry-policies.sh` follows `fn0Cloud:dbBackend` for read and check modes. Its historical `--apply` migration must be completed while Turso is authoritative; after dodb cutover, `--apply` fails before any database or Signy write. Normal post-cutover operator paths never consult Turso. Direct Turso access is restricted to explicit migration and rollback tooling, including `scripts/run-dodb-migration.sh` and its `fn0-db-migrate` source reader. The obsolete `scripts/scale-config.sh` was removed because it had no active consumer and the worker pool no longer has automatic CPU autoscaling.

The native `fn0-db-ops` binary is a narrow operator interface fixed to project `fn0-control`. The operator session builds its Linux ARM64 binary, opens OCI Bastion Port Forwarding to dodb SSH, uploads the binary to a random `/tmp/fn0-db-ops.*` path, and reuses that SSH tunnel for the rest of the parent script. The binary runs on the private dodb VM and connects to `127.0.0.1:18445` over QUIC with server name `dodb.internal` and `/etc/dodb/server.crt`. The Mac never tunnels QUIC through Bastion TCP. The helper removes the remote binary, tunnel, Bastion session, and local temporary files when the parent script exits.

New worker and worker-agent instances receive `DODB_ADDR`, `DODB_SERVER_NAME`, and `DODB_ROOT_CERT_PEM_BASE64`. These values are materialized in production Pulumi stack configuration from the current dodb outputs to avoid a Pulumi dependency cycle. Resync all three runtime values whenever dodb is replaced or its certificate rotates. Only the public root certificate is distributed; the dodb private key is not sent to workers.

The future worker instance configuration uses one `VM.Standard.A1.Flex` worker with 1 OCPU, 6 GB memory, and a 50 GB boot volume. Collecty stores its queue at `/var/lib/collecty` on the boot filesystem. It has no separate collecty block volume and no automatic CPU autoscaling. Existing live workers are not replaced by source or stack configuration changes alone.

Exact verification separately rescans current Turso and dodb rows in `(pk, sk)` order, comparing every key and payload byte. It reports source/destination row and byte totals plus missing, extra, and different row counts. Mismatch output contains keys only, with a default sample limit of 20. Full migration discovers the active project set before copying and rediscovers it after all copies; a changed set stops migration before verification. The exact verification then rereads current source data, so source changes during copying are detected as mismatches.

## Remote runner

`scripts/run-dodb-migration.sh inventory` runs a native local `fn0-db-migrate` and reads only `forteDbGroupToken` and `forteDbHostSuffix` from Pulumi. It does not open Bastion, SSH, or a dodb connection. dodb uses QUIC/UDP, while OCI Bastion is a TCP SSH tunnel, so the local machine does not tunnel the dodb protocol. The remote commands open OCI Bastion Port Forwarding to dodb SSH, then the VM connects to `127.0.0.1:18445` over QUIC and to Turso over HTTPS through NAT.

```sh
scripts/run-dodb-migration.sh inventory
scripts/run-dodb-migration.sh transport-check
scripts/run-dodb-migration.sh verify
scripts/run-dodb-migration.sh migrate --apply
```

The remote runner builds a Linux ARM64 binary and uses the OCI-returned `ssh-metadata.command` for Bastion Port Forwarding. It streams a gzip-compressed binary through ordinary SSH stdin, verifies its SHA-256 on the VM, and atomically installs it at `/home/opc/.cache/fn0/db-migrate/<sha256>/fn0-db-migrate`. Matching cache entries skip upload and remain on the VM. No SCP or SFTP is used. Remote SSH commands use batch mode, a 10 second connect timeout, and keepalives every 15 seconds with three missed replies allowed.

`transport-check` reads only the Bastion ID, worker SSH private key, and dodb private IP. It requires no Turso credentials and only checks or uploads the cached executable, runs `fn0-db-migrate --help`, and lists and removes only exact legacy runner temporary names. It performs no database operation. The normal remote `migrate` and `verify` commands send their arguments and shell-escaped Turso environment file through SSH stdin. The Turso token stays out of command arguments and process listings. The server certificate is copied to a unique temporary path readable by `opc`. Remote env, argument, and certificate files, the Bastion tunnel and session, and local temporary files are removed when the runner exits.

The `fn0-db-ops` operator binary is also cached by SHA-256 under `/home/opc/.cache/fn0/db-ops/` and streamed with gzip over SSH stdin. The cache is retained when the operator session closes; no SCP or SFTP transfer is used for this binary.

This runner was added for the future maintenance operation. It must not be run against production until the cutover prerequisites below are met.

## Future cutover procedure

The following sequence is documentation only; implementation and tooling work does not execute it.

1. Enter maintenance mode.
2. Block external requests.
3. Stop or background-disable every Turso writer.
4. Confirm no database writer remains.
5. Run `scripts/run-dodb-migration.sh inventory`.
6. Run `scripts/run-dodb-migration.sh migrate --apply`; migration automatically performs exact verification.
7. Run `scripts/run-dodb-migration.sh verify` separately.
8. Only after exact verification, update worker and control dodb configuration.
9. Deploy or restart the relevant services.
10. Smoke test the intended application path.
11. Reopen traffic.
12. Keep Turso intact as the rollback source until the confidence window ends.

Before traffic reopens, Turso remains authoritative and a rollback is straightforward because no post-cutover writes have made Turso stale. After traffic reopens and dodb accepts writes, Turso is stale and a simple rollback to Turso is unsafe. This tooling does not implement dual-write or change-data capture. Do not delete Turso immediately after cutover.
