use serde::{Deserialize, Serialize};

use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct DedicatedHostRecord {
    pub addr: String,
    pub port: u16,
    #[serde(default)]
    pub transport: DwsTransport,
    #[serde(default)]
    pub http_host: Option<String>,
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DwsTransport {
    #[default]
    Quic,
    WebSocket,
}

impl DocDb {
    pub async fn list_dedicated_hosts(&self) -> Result<Vec<(String, DedicatedHostRecord)>> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT pk, value FROM docs WHERE pk LIKE 'dws-host:%' AND sk = 0",
                libsql::params!(),
            )
            .await?;

        let mut hosts = Vec::new();
        while let Some(row) = rows.next().await? {
            let pk: String = row.get(0).unwrap();
            let json: String = row.get(1).unwrap();
            let host_id = pk.strip_prefix("dws-host:").unwrap_or(&pk).to_string();
            let record: DedicatedHostRecord = serde_json::from_str(&json).unwrap();
            hosts.push((host_id, record));
        }

        Ok(hosts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_dedicated_hosts() {
        let db = DocDb::new_test_db().await.unwrap();
        let conn = db.db.connect().unwrap();

        conn.execute(
            "INSERT INTO docs (pk, sk, value) VALUES ('dws-host:my-server-1', 0, ?)",
            libsql::params![
                serde_json::to_string(&DedicatedHostRecord {
                    addr: "1.2.3.4".to_string(),
                    port: 10000,
                    transport: DwsTransport::Quic,
                    http_host: None,
                })
                .unwrap()
            ],
        )
        .await
        .unwrap();

        conn.execute(
            "INSERT INTO docs (pk, sk, value) VALUES ('dws-host:my-server-2', 0, ?)",
            libsql::params![
                serde_json::to_string(&DedicatedHostRecord {
                    addr: "my-host.example.com".to_string(),
                    port: 10000,
                    transport: DwsTransport::WebSocket,
                    http_host: Some("my-host-http.example.com".to_string()),
                })
                .unwrap()
            ],
        )
        .await
        .unwrap();

        let hosts = db.list_dedicated_hosts().await.unwrap();
        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[0].0, "my-server-1");
        assert_eq!(hosts[0].1.addr, "1.2.3.4");
        assert_eq!(hosts[1].0, "my-server-2");
        assert_eq!(hosts[1].1.addr, "my-host.example.com");
    }

    #[tokio::test]
    async fn test_list_dedicated_hosts_empty() {
        let db = DocDb::new_test_db().await.unwrap();
        let hosts = db.list_dedicated_hosts().await.unwrap();
        assert!(hosts.is_empty());
    }
}
