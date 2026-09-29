# Cloudflare Access and Tunnel verification

This check is deployment-specific because the hostname, Access application and
Tunnel credentials belong to the operator. It exercises the same queue and
sender used in production; it does not bypass collecty with a direct signy
request.

1. Put signy behind a Cloudflare Tunnel whose public hostname is protected by
   a service-token policy. Keep signy's listener on its private HTTP address.
2. Start collecty with the hostname and both service-token values supplied by
   the deployment secret store:

   ```text
   COLLECTY_SIGNY_URL=https://signy.example.test
   COLLECTY_SIGNY_ACCESS_CLIENT_ID=<service-token client id>
   COLLECTY_SIGNY_ACCESS_CLIENT_SECRET=<service-token client secret>
   ```

   Do not put the values in command history or a world-readable environment
   file. Collecty sends them only as `CF-Access-Client-Id` and
   `CF-Access-Client-Secret` on the signy request.
3. Send one valid OTLP/HTTP protobuf export to collecty's `/v1/logs` route and
   record the HTTP acknowledgement. Confirm signy receives the corresponding
   resource through the Tunnel.
4. Stop signy while leaving collecty and the Tunnel running. Send another
   export and record that collecty still acknowledges the intake while a
   sealed segment remains in `COLLECTY_DATA_DIR/queue/`.
5. Start signy again. Confirm the queued segment is delivered and retired, the
   resource appears once in signy, and the queue returns to its pre-outage
   state. A retry log may include the URL and status but must not include either
   service-token value.

The transport unit tests cover the HTTP path, a certificate and hostname that
are explicitly trusted by the test client, untrusted and mismatched
certificates, exact Access header placement, and redaction of credentials from
delivery errors. Record the deployment hostname, outage interval, queue
segment before and after recovery, and signy resource identity with the run
for an auditable E2E result.
