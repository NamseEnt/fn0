# Cloudflare Access and Tunnel E2E result

Date: 2026-09-14
Commit under test: `55c0d214aecce2e8845474ed791e31a1e26eef9d`
Host: `192.168.0.10`

## Deployment

- Dedicated hostname: `signy-e2e-20260914011608.fn0.dev`
- Dedicated R2 bucket and run prefix: `obsy-signy-e2e-20260914011608/run-20260914011608`
- Bucket Lock rules covered `run-20260914011608/signy/catalog/` and `run-20260914011608/smoke/catalog/` for seven days.
- R2 catalog backend smoke test: passed.
- Image digests: signy `sha256:8cfd97dc51bdc5086e0b12721aadca8235aab92730ee65f6862750b097327517`; collecty `sha256:4946ba111f6e2615b24a8ce0c2dd9d526589fa5059b719b3bf4a10b4bd1b0497`.

## Results

- Unauthenticated `GET https://signy-e2e-20260914011608.fn0.dev/ready`: `401`.
- The same request with the Access service-token headers: `200`.
- The load generator sent through collecty, not directly to signy. It received `41` intake acknowledgements, all HTTP `200`, with no dropped resources.
- signy was stopped at `2026-09-14T01:41:08Z` and restarted at `2026-09-14T01:41:21Z`; readiness returned at `2026-09-14T01:41:28Z`.
- During the outage collecty returned intake acknowledgements while its sender received Cloudflare `502` responses. The queue report reached `queued_bytes=972`, `segments=4`, and `retries=5`.
- After signy recovered, the queue report returned to `queued_bytes=0`, `segments=3`, and the sent-segment count advanced from `3` to `5`.
- signy performed a graceful shutdown, claimed a new object-store writer epoch, restored manifest generation `6` with `3` parts, and resumed listening.
- The final tenant query returned `44` rows. The pre-outage run had already persisted `3` rows; the outage run accepted `41`, so the observed total is consistent with one delivery per accepted event. All `44` timestamps were distinct.
- Full signy and collecty logs contained zero matches for the Access secret, R2 secret, or Tunnel token values.

## Harness note

The load harness reported `FAIL` for its numeric verdict because the deliberate outage raised response p95 to `394.8 ms` against its normal `250 ms` target and no server memory source was configured. Its behavioral fields still recorded delivery, zero drops, a settled collector drain, and a recovered WAL backlog. The transport and outage-recovery acceptance criteria in `ACCESS_TUNNEL_E2E.md` passed.

## Cleanup

The dedicated signy/collecty containers, host files, data directories, Tunnel, DNS record, Access application, and Access service token were removed. The R2 bucket remains because its catalog prefixes are Bucket-Locked; it contains only this run's small test dataset. The bucket-scoped R2 API token could not be revoked with the available operator or bootstrap token (`403 Account API Tokens Write`); revoke it from a Super Administrator account before reusing or deleting the bucket.
