# Production server

## Binary

Build the standalone production executable with:

```bash
cargo build --release -p dodb-server
```

The executable serves dodb's existing raw QUIC protocol.

## Command line

The server requires `--data-dir`, `--tls-cert`, and `--tls-key`.

| Option | Default | Description |
| --- | --- | --- |
| `--listen` | `127.0.0.1:18445` | QUIC listen address |
| `--data-dir` | required | Directory for tenant shard files |
| `--tls-cert` | required | PEM certificate chain |
| `--tls-key` | required | PEM private key |
| `--max-open-shards` | `1024` | Maximum open tenant shards |
| `--max-connections` | `8` | Maximum concurrent QUIC connections |
| `--max-concurrent-streams` | `64` | Bidirectional request streams advertised per connection |
| `--max-concurrent-requests` | `64` | Global active service requests |

All limits must be nonzero.

## Example

```bash
dodb-server \
  --listen 0.0.0.0:18445 \
  --data-dir /var/lib/dodb \
  --tls-cert /etc/dodb/server.crt \
  --tls-key /etc/dodb/server.key
```

## Storage

The data directory is required and is created by `LocalTenantService` when it
does not exist. The server uses the existing storage, database, and coordinator
defaults. Preserve the storage durability behavior expected by dodb.

## Shutdown

On Unix, SIGTERM and SIGINT stop accepting new QUIC connections. The server
drains active connections and requests, then shuts down the tenant service and
its storage coordinators before exiting. On non-Unix platforms, Ctrl+C requests
the same graceful shutdown.

## Network security

The default bind address is `127.0.0.1:18445`.

Binding to `0.0.0.0` must only be used behind a private network or firewall.
TLS authenticates and encrypts the server connection. dodb currently does not
authenticate clients or authorize tenants. Do not expose dodb directly to the
public Internet.

## Backup

Use the storage-level backup and restore contract documented in
[`storage-backup.md`](storage-backup.md). dodb does not create or transfer
application-level backup snapshots.
