# TODO

## Send to signy through Cloudflare Access

Goal: let a worker-local collecty reach signy through a Cloudflare Tunnel
without another local proxy.

- [x] Accept `https://` in `COLLECTY_SIGNY_URL` with normal certificate and
      hostname verification; keep `http://` for trusted private networks.
- [x] Add secret environment settings for the Cloudflare Access client ID and
      secret, and send them only as `CF-Access-Client-Id` and
      `CF-Access-Client-Secret` on signy requests.
- [x] Keep the current delivery contract unchanged: the collect route, sender
      headers, timeouts, retryable responses, refused payloads and durable queue
      retirement must behave identically over HTTP and HTTPS.
- [x] Prove HTTP compatibility, verified HTTPS, rejected certificates, Access
      header injection and secret redaction with transport tests.
- [x] Verify one queued export through Cloudflare Access and Tunnel into signy,
      including recovery after a temporary signy outage. The operator procedure
      is documented in [`docs/ACCESS_TUNNEL_E2E.md`](docs/ACCESS_TUNNEL_E2E.md);
      the 2026-09-14 result is recorded in
      [`docs/ACCESS_TUNNEL_E2E_RESULT_2026-09-14.md`](docs/ACCESS_TUNNEL_E2E_RESULT_2026-09-14.md).

Non-goals: TLS on collecty's ingest listener, generic arbitrary request headers,
or changing signy's authentication model.
