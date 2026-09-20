# Production Signy Telemetry

The fn0 production telemetry path is:

`fn0 workload -> worker OTLP -> collecty durable queue -> Signy -> Cloudflare R2`

The active worker pool has one worker. Collecty listens on `127.0.0.1:4318`,
and the worker exports its own telemetry and every forwarded guest export
there. Signy listens
on `127.0.0.1:3100` on `192.168.0.10`. External access is
`https://signy.fn0.dev`, protected by a Cloudflare Access service token and
served through the Signy Tunnel.

## Pinned production resources

- collecty image:
  `ocir.ap-osaka-1.oci.oraclecloud.com/axhyjd4qpgot/fn0-worker-rx1ebeyn@sha256:249ba93f391d71db4bd4ae8826eac12cf5c80e57b878c6672cbb46014da86866`
- Signy image:
  `ocir.ap-osaka-1.oci.oraclecloud.com/axhyjd4qpgot/fn0-worker-rx1ebeyn@sha256:8212e26d9e46e5d52ff9ee8c498673959347f5cb0a705a7057f0a3b78eed357d`
- Obsy source revision for the collecty image: `406a55fca23c433a05c772510e96d9c6ebb5e0e4`
- Obsy source revision for the Signy image: `4da304fb9c1ada8d8871076fae92bae402f39317`
  (built on the x86_64 Signy node from `signy/Dockerfile`)
- R2 bucket: `fn0-signy-u35twkcf`
- R2 prefix: `fn0/signy`
- R2 catalog lock: seven days
- Access application: `signy.fn0.dev`
- Tunnel id: `cfdc38c0-48b9-43db-848a-a5f442a695ec`

The image references and the telemetry configuration version are stored in
`infra/cloud/Pulumi.prod.yaml`. Worker cloud-init and the node setup script
consume those values, so a redeploy cannot silently select a mutable tag.

The node is running the pinned Signy image above. It was built on the x86_64
Signy node from the source revision listed above.

## Deployment

Apply the Pulumi stack before configuring the node:

```sh
cd infra/cloud
npm run build
npx tsc --noEmit
pulumi up
```

Configure or refresh the permanent Signy node with:

```sh
scripts/setup-signy-node-remote.sh --ssh namse@192.168.0.10
```

The script writes root-only secret files, starts the digest-pinned Signy
container, configures the Tunnel, removes obsolete metrics tunnel and backup
units, and verifies R2 health. It does not create a platform tenant policy;
control owns explicit project policies.

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
raw paths and raw error messages go to logs only. Signy container logs rotate at five
100 MiB files. At the verification time the R2 bucket contained 1,281 objects
using about 2.1 MiB; this includes only the current rollout and verification
data. VictoriaMetrics
and its old backup/tunnel units are no longer part of the deployment.

## Verification record

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

If an outage test is repeated, stop only the `signy` container on
`192.168.0.10`, issue a small number of normal `fn0.dev` requests, observe
`/var/lib/collecty` growth and collecty's retry logs, start Signy, then wait
for the queue to drain before declaring recovery.
