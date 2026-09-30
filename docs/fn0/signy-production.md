# Production Signy Telemetry

The fn0 production telemetry path is:

`fn0 workload -> worker OTLP -> collecty durable queue -> Signy on the DODB OCI VM -> Cloudflare R2`

The active worker pool has one worker. Collecty listens on `127.0.0.1:4318`,
and the worker exports its own telemetry and every forwarded guest export
there. Production Signy runs on the existing DODB OCI VM, reached by operators
through OCI Bastion port forwarding. Its host port is published only on
`127.0.0.1:3100`. The external endpoint remains `https://signy.fn0.dev`,
protected by the existing Cloudflare Access service tokens and served through
the existing Signy Tunnel.

## Migration status

The migration completed on 2026-09-30. The same-revision ARM64 image from
`namse/obsy` is running on DODB. The existing Tunnel is healthy and connected;
external readiness and tenant queries succeed. The former standalone node and
its storage have not been deleted. Data acknowledged by the old process but
not flushed to R2 may remain only in its inaccessible local WAL.

## Pinned production resources

- collecty image:
  `ocir.ap-osaka-1.oci.oraclecloud.com/axhyjd4qpgot/fn0-worker-rx1ebeyn@sha256:10e14acafb8d7a367b0d0b3bf9269fa925ff788e23dbeb76326725f2854e6620`
- Signy image:
  `ocir.ap-osaka-1.oci.oraclecloud.com/axhyjd4qpgot/fn0-worker-rx1ebeyn@sha256:a7ac0dc5f1e67f9521e9d7b7554d9d2944ae86b3e037219fa508c727e8d310b5`
- Obsy source revision for the collecty image: `c6c7d5c8fada9f05be31c418a5e673fbe25476a9`
- Signy source repository: `namse/obsy`
- Signy source revision: `4da304fb9c1ada8d8871076fae92bae402f39317`
- Signy image platform: `linux/arm64`
- Signy build branch metadata: `production-arm64-migration`
- R2 bucket: `fn0-signy-u35twkcf`
- R2 prefix: `fn0/signy`
- R2 catalog lock: seven days
- Access application: `signy.fn0.dev`
- Tunnel id: `cfdc38c0-48b9-43db-848a-a5f442a695ec`

The image references and the telemetry configuration version are stored in
`infra/cloud/Pulumi.prod.yaml`. Worker cloud-init and the DODB deploy script
consume those values, so a redeploy cannot silently select a mutable tag.

The Signy image was built from the exact production source revision in the old
`namse/obsy` repository using `signy/Dockerfile`. Its immutable child manifest
digest is pinned in Pulumi config; no mutable tag is used for the service.

## Deployment

The existing Pulumi stack owns the Signy R2 bucket, Tunnel, hostname, Access
application, and service tokens. They were reused without recreation or
rotation. Pulumi config pins the Signy image digest. The image-ref output was
targeted separately; the Signy Tunnel, R2 resources, DODB instance, and worker
image were not changed.

```sh
cd infra/cloud
npm run build
npx tsc --noEmit
```

Deploy through the existing OCI Bastion and the DODB service path with:

```sh
scripts/deploy-signy-on-dodb.sh
```

The deploy script refuses to install the services unless the configured image
is digest-pinned and the pulled image is ARM64. It writes root-only secret
files, starts `fn0-signy.service` and `fn0-signy-tunnel.service`, verifies R2
health, platform tenant metric and log queries, the existing Tunnel, and the
external Access path. Startup uses the required `RUST_LOG=signy=warn`, so
info-level restore messages are suppressed; readiness, remote R2 health, and
queries against restored history verify successful startup. The installer does
not create a platform tenant policy; control owns explicit project policies.

The standalone setup scripts are legacy utilities. Current DODB operations use
`scripts/deploy-signy-on-dodb.sh` through OCI Bastion.

## Migration verification

On 2026-09-30, the DODB VM reported `dodb.service`, `fn0-collecty.service`,
`fn0-signy.service`, and `fn0-signy-tunnel.service` active. Signy `/ready`
returned HTTP 200 locally and externally; `signy_remote_healthy` was `1`.
The existing Tunnel reported healthy with eight connections. Platform tenant
instant and range metric queries returned data, and an eight-day logs query
returned ten rows. `signy_query_quota_rejected_total` and
`signy_query_errors_total` were both zero.

The worker and DODB Collecty durable queues returned to their 20-byte identity
baseline. No queue file was cleared or reset. Ops Console DODB and host history,
worker history, and Collecty queue/segment history all loaded for 15 minutes,
one hour, six hours, 24 hours, and seven days. At the final check, its top-level
Worker health still reported “no reporting worker instance has its manifest
loaded”, while a direct Signy instant query returned the latest worker gauge as
`manifest_loaded=1`, `draining=0`, and the worker's local `/ready` endpoint
returned HTTP 200. This dashboard-health discrepancy remains under
investigation; the history panels and telemetry pipeline were available.

The targeted Pulumi preview for the Ops Console history update contained one
`WorkersScript` update and 147 unchanged resources. The broad preview also
reported the accepted R2 event-notification `missing ID` drift; that resource
was not targeted or applied.

The required warning-level Signy log setting suppresses info-level messages
that would separately name writer-epoch claim and each catalog or manifest
restore. Startup completed readiness, remote-storage, and restored-data query
checks without a panic or error log.

## Cost and retention controls

Collecty uses a 1 GiB queue cap per worker, 8 MiB segments, a two-second
segment age, warning-level logs, a 30-second send timeout, and no journald
collection. Host metrics are sampled once per minute. Signy runs with
`RUST_LOG=signy=warn`. The queue is mounted on
the worker's 50 GiB volume and is the recovery buffer when Signy is unavailable.

Signy declares a 2 GiB memory budget, an 8 GiB cache limit, a 1 GiB WAL
backlog limit, and a 4 GiB minimum free-disk floor. Retention and storage
limits are explicit per-tenant policies; there is no global or platform
retention fallback.

Signy flushes at most once a minute (`SIGNY_FLUSH_MAX_INTERVAL=60s`), so data
reaches R2 up to a minute after it is written to the local WAL. Fewer flushes
mean fewer parts and fewer catalog commits. Log, trace and metric parts are
compacted by size tier, so a part is only ever rewritten with parts of its own
size. Orphaned part objects are collected every hour
(`SIGNY_ORPHAN_GC_INTERVAL=1h`) whether or not retention expired anything, in
passes bounded by `SIGNY_ORPHAN_GC_MAX_RUNTIME`,
`SIGNY_ORPHAN_GC_MAX_SCANNED_OBJECTS`, `SIGNY_ORPHAN_GC_MAX_DELETED_OBJECTS`
and `SIGNY_ORPHAN_GC_MAX_DELETED_BYTES` that resume where the last one
stopped; `SIGNY_ORPHAN_GC_DRY_RUN=true` reports what a pass would delete
without deleting it. Catalog commits and snapshots older than eight days
(`SIGNY_CATALOG_PRUNE_MIN_AGE=8d`, one day past the seven-day Bucket Lock) are
deleted once two newer snapshots verify, on their own schedule rather than
behind orphan collection.

On the worker, 1% of requests are traced and background loops are never traced.
Server errors, requests that ran out of time and failed static page
generations are logged; a slow request that succeeded is not.
Request metrics carry the project, the route template, and a bounded outcome;
raw paths and raw error messages go to logs only. The former Docker setup
rotated Signy container logs at five 100 MiB files. The DODB Podman unit will
cap each container log at 100 MiB. At the historical verification time the R2
bucket contained 1,281 objects using about 2.1 MiB; this includes only the
current rollout and verification data. VictoriaMetrics
and its old backup/tunnel units are no longer part of the deployment.

## Historical verification record

The production checks completed on 2026-09-14:

1. `https://signy.fn0.dev/ready` returned `401` without Access headers and
   `200` with the configured service token.
2. The worker services `fn0-collecty`, `fn0-worker-agent`, and
   `fn0-worker-proxy` were all active. Collecty ran the pinned digest above.
3. Requests to `https://fn0.dev` generated records queried from Signy with
   `X-Tenant-Id: fn0`; returned records carried `project_id=fn0-control` and
   `service_name=fn0-worker`.
4. Signy was stopped while real `fn0.dev` requests continued. Collecty logged
   `502 Bad Gateway` responses and retained telemetry on disk; the queue grew
   to 19,724 bytes with non-empty metric and trace segments.
5. Signy was restarted. After the retry backoff elapsed, the queue returned to
   its 20-byte identity baseline. Signy then reported
   `signy_remote_healthy 1`, `signy_ingest_errors_total 0`, and successful
   flushes. A subsequent `fn0.dev` request was visible in Signy's query path.

This verification record describes the former standalone deployment as of
2026-09-14. Do not use its retired node address for production operations.

The operator console's telemetry health signals and production checks are
documented in [operations-dashboard.md](operations-dashboard.md).
