use super::*;
use crate::args::CloudflareDnsProviderArgs;
use std::net::IpAddr;

const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

pub struct CloudflareDnsProvider {
    client: reqwest::Client,
    zone_id: String,
    asterisk_domain: String,
    saas_fallback_domain: String,
    api_token: String,
    api_url: String,
}

impl CloudflareDnsProvider {
    pub fn new(args: CloudflareDnsProviderArgs, api_url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .local_address("::".parse().ok())
                .build()
                .unwrap(),
            zone_id: args.zone_id,
            asterisk_domain: args.asterisk_domain,
            saas_fallback_domain: args.saas_fallback_domain,
            api_token: args.api_token,
            api_url: api_url.unwrap_or_else(|| "https://api.cloudflare.com/client/v4".to_string()),
        }
    }

    fn target_names(&self) -> [&str; 2] {
        [&self.asterisk_domain, &self.saas_fallback_domain]
    }

    async fn list_records_for(&self, name: &str) -> color_eyre::Result<Vec<Record>> {
        let url = format!("{}/zones/{}/dns_records", self.api_url, self.zone_id);
        let params = [("per_page", "5000000"), ("name.exact", name)];

        #[derive(Debug, serde::Deserialize)]
        struct CloudflareDnsRecordsResponse {
            success: bool,
            result: Option<Vec<RecordResponse>>,
            #[allow(dead_code)]
            errors: serde_json::Value,
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
            eprintln!("Failed to list records for {name}: {response:?}");
            return Err(color_eyre::eyre::eyre!("Failed to list records for {name}"));
        }

        Ok(response
            .result
            .unwrap_or_default()
            .into_iter()
            .filter(|record| {
                record.r#type == "A" || record.r#type == "AAAA" || record.r#type == "CNAME"
            })
            .map(|record| Record {
                name: name.to_string(),
                content: record.content,
                record_type: record.r#type,
                id: record.id,
            })
            .collect())
    }
}

fn addr_to_record_type(addr: &str) -> &'static str {
    match addr.parse::<IpAddr>() {
        Ok(IpAddr::V4(_)) => "A",
        Ok(IpAddr::V6(_)) => "AAAA",
        Err(_) => "CNAME",
    }
}

impl DnsProvide for CloudflareDnsProvider {
    async fn sync_addrs(&self, addrs: BTreeSet<String>) -> color_eyre::Result<()> {
        let names = self.target_names();

        let mut old_records: Vec<Record> = Vec::new();
        for name in &names {
            old_records.extend(self.list_records_for(name).await?);
        }

        #[derive(serde::Serialize)]
        struct Body<'a> {
            deletes: Vec<Delete<'a>>,
            posts: Vec<Post<'a>>,
        }

        #[derive(serde::Serialize)]
        struct Delete<'a> {
            id: &'a str,
        }

        #[derive(serde::Serialize)]
        struct Post<'a> {
            name: &'a str,
            ttl: usize,
            r#type: &'static str,
            content: &'a str,
            proxied: bool,
        }

        let mut posts: Vec<Post> = Vec::new();
        for name in &names {
            for addr in &addrs {
                let record_type = addr_to_record_type(addr);
                let already = old_records
                    .iter()
                    .any(|r| r.name == *name && r.content == *addr && r.record_type == record_type);
                if !already {
                    posts.push(Post {
                        name,
                        ttl: 60,
                        r#type: record_type,
                        content: addr,
                        proxied: true,
                    });
                }
            }
        }

        let deletes: Vec<Delete> = old_records
            .iter()
            .filter(|record| {
                addrs.iter().all(|addr| {
                    let record_type = addr_to_record_type(addr);
                    !(record.content == *addr && record.record_type == record_type)
                })
            })
            .map(|record| Delete {
                id: record.id.as_str(),
            })
            .collect();

        if posts.is_empty() && deletes.is_empty() {
            return Ok(());
        }

        let response = self
            .client
            .post(format!(
                "{}/zones/{}/dns_records/batch",
                self.api_url, self.zone_id
            ))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_token))
            .body(serde_json::to_string(&Body { deletes, posts })?)
            .timeout(DEFAULT_TIMEOUT)
            .send()
            .await?
            .text()
            .await?;

        println!("cloudflare sync_addrs dns_records/batch Response: {response}");

        Ok(())
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
struct Record {
    name: String,
    content: String,
    record_type: String,
    id: String,
}
