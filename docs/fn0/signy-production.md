# Production Signy Telemetry

The fn0 production telemetry path is:

`fn0 workload -> worker OTLP -> collecty durable queue -> Signy on the DODB OCI VM -> Cloudflare R2`

The active worker pool has one worker. Collecty listens on `127.0.0.1:4318`,
and the worker exports its own telemetry and every forwarded guest export
there. The production Signy host target is the existing DODB OCI VM, managed
through OCI Bastion port forwarding. Signy will listen on `127.0.0.1:3100`.
The external endpoint remains `https://signy.fn0.dev`, protected by the
existing Cloudflare Access service tokens and served through the existing
Signy Tunnel; that path is currently unavailable pending the migration.

## Migration status

The former standalone Signy node is inaccessible and its Tunnel is down. The
DODB-hosted Signy services have not yet been installed. The current pinned
production image is x86_64 and cannot run on the ARM64 DODB VM. The documented
production source revision `4da304fb9c1ada8d8871076fae92bae402f39317` is not
available in this checkout or the configured source repository, so the
required same-revision ARM64 image has not been built. The DODB deploy script
checks the pulled image architecture before installing service files and will
refuse the current image.

After the matching ARM64 image is available, use
`scripts/deploy-signy-on-dodb.sh`. Do not operate the old standalone node as a
production writer. The old node and its storage have not been deleted. Data
acknowledged by the old process but not flushed to R2 may remain only in its
unavailable local WAL.

## Pinned production resources

- collecty image:
  `ocir.ap-osaka-1.oci.oraclecloud.com/axhyjd4qpgot/fn0-worker-rx1ebeyn@sha256:10e14acafb8d7a367b0d0b3bf9269fa925ff788e23dbeb76326725f2854e6620`
- Signy image:
  `ocir.ap-osaka-1.oci.oraclecloud.com/axhyjd4qpgot/fn0-worker-rx1ebeyn@sha256:8212e26d9e46e5d52ff9ee8c498673959347f5cb0a705a7057f0a3b78eed357d`
- Obsy source revision for the collecty image: `c6c7d5c8fada9f05be31c418a5e673fbe25476a9`
- Obsy source revision for the Signy image: `4da304fb9c1ada8d8871076fae92bae402f39317`
  (built on the x86_64 Signy node from `obsy/signy/Dockerfile`)
- R2 bucket: `fn0-signy-u35twkcf`
- R2 prefix: `fn0/signy`
- R2 catalog lock: seven days
- Access application: `signy.fn0.dev`
- Tunnel id: `cfdc38c0-48b9-43db-848a-a5f442a695ec`

The image references and the telemetry configuration version are stored in
`infra/cloud/Pulumi.prod.yaml`. Worker cloud-init and the DODB deploy script
consume those values, so a redeploy cannot silently select a mutable tag.

The pinned Signy image above is the former standalone-node image. It was built
on the x86_64 Signy node from the source revision listed above and is not
compatible with the ARM64 DODB VM.

## Deployment

The existing Pulumi stack already owns the Signy R2 bucket, Tunnel, hostname,
Access application, and service tokens. Do not recreate or rotate them. A
Pulumi apply is not required for the DODB-hosted service deployment unless a
future preview identifies a necessary output-only wiring change.

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
health and manifest restoration, checks platform tenant queries, and verifies
the existing Tunnel and external Access path. It does not create a platform
tenant policy; control owns explicit project policies. Until the ARM64 image
gate passes, production Signy remains unavailable.

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
