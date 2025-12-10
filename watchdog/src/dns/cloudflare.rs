use super::*;
use crate::*;
use std::{env, net::IpAddr};

pub struct CloudflareDns {
    client: reqwest::Client,
    zone_id: String,
    /// ex) *.mydomain.com
    asterisk_domain: String,
    api_token: String,
}

impl CloudflareDns {
    pub async fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .local_address("[::]:0".parse().ok())
                .build()
                .unwrap(),
            zone_id: env::var("CLOUDFLARE_ZONE_ID").expect("env var CLOUDFLARE_ZONE_ID is not set"),
            asterisk_domain: env::var("CLOUDFLARE_ASTERISK_DOMAIN")
                .expect("env var CLOUDFLARE_ASTERISK_DOMAIN is not set"),
            api_token: env::var("CLOUDFLARE_API_TOKEN")
                .expect("env var CLOUDFLARE_API_TOKEN is not set"),
        }
    }
    async fn list_records(&self) -> color_eyre::Result<Vec<Record>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            self.zone_id
        );
        let params = [
            ("per_page", "5000000"),
            ("name.exact", self.asterisk_domain.as_str()),
        ];

        #[derive(Debug, serde::Deserialize)]
        struct CloudflareDnsRecordsResponse {
            success: bool,
            result: Vec<RecordResponse>,
        }

        #[derive(Debug, serde::Deserialize)]
        struct RecordResponse {
            r#type: String,
            content: String,
            id: String,
        }

        let text = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .query(&params)
            .timeout(DEFAULT_TIMEOUT)
            .send()
            .await?
            .text()
            .await?;

        let response: CloudflareDnsRecordsResponse = serde_json::from_str(&text)?;

        if !response.success {
            eprintln!("Failed to list records: {response:?}");
            return Err(color_eyre::eyre::eyre!("Failed to list records"));
        }

        Ok(response
            .result
            .into_iter()
            .filter(|record| record.r#type == "A" || record.r#type == "AAAA")
            .map(|record| Record {
                ip: record.content.parse().unwrap(),
                id: record.id,
            })
            .collect())
    }
}

impl Dns for CloudflareDns {
    fn sync_ips<'a>(
        &'a self,
        ips: Vec<IpAddr>,
    ) -> Pin<Box<dyn Future<Output = color_eyre::Result<()>> + 'a + Send>> {
        Box::pin(async move {
            let old_records = self.list_records().await?;

            let new_ips = ips
                .iter()
                .filter(|ip| old_records.iter().all(|record| record.ip != **ip));

            let deleted_ips = old_records
                .iter()
                .filter(|record| ips.iter().all(|ip| record.ip != *ip));

            #[derive(serde::Serialize)]
            struct Body<'a> {
                deletes: Vec<&'a str>,
                posts: Vec<BodyRecord<'a>>,
            }

            #[derive(serde::Serialize)]
            struct BodyRecord<'a> {
                name: &'a str,
                ttl: usize,
                r#type: &'static str,
                content: String,
                proxied: bool,
            }

            self.client
                .post(format!(
                    "https://api.cloudflare.com/client/v4/zones/{}/dns_records/batch",
                    self.zone_id
                ))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", self.api_token))
                .body(serde_json::to_string(&Body {
                    deletes: deleted_ips.map(|record| record.id.as_str()).collect(),
                    posts: new_ips
                        .map(|ip| BodyRecord {
                            name: &self.asterisk_domain,
                            ttl: 60,
                            r#type: match ip {
                                IpAddr::V4(_) => "A",
                                IpAddr::V6(_) => "AAAA",
                            },
                            content: ip.to_string(),
                            proxied: false,
                        })
                        .collect(),
                })?)
                .timeout(DEFAULT_TIMEOUT)
                .send()
                .await?
                .text()
                .await?;

            Ok(())
        })
    }
}
struct Record {
    ip: IpAddr,
    id: String,
}
