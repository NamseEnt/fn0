use color_eyre::eyre::{Result, eyre};
use doc_db::DocDb;

use crate::{
    args::*,
    deployment_cache::DeploymentCache,
    dns::{DnsProvider, cloudflare::CloudflareDnsProvider},
    host_provider::{HostProvider, oci_container::OciContainerInstanceHostProvider},
    site::Site,
};

pub struct HqArgsParsed {
    pub sites: Vec<Site>,
    pub deployment_cache: DeploymentCache,
}

impl HqArgs {
    pub async fn parse() -> Result<HqArgsParsed> {
        let path = std::env::var("HQ_ARGS_PATH").unwrap_or_else(|_| "hq-args.json".to_string());

        let content = std::fs::read_to_string(&path)
            .map_err(|e| eyre!("Failed to read config file at {}: {}", path, e))?;

        let args: HqArgs = serde_json::from_str(&content)
            .map_err(|e| eyre!("Failed to parse config file: {}", e))?;

        let doc_db = DocDb::new(args.doc_db.url, args.doc_db.token).await?;
        let deployment_cache = DeploymentCache::new(doc_db.clone()).await?;

        let sites = args
            .sites
            .into_iter()
            .map(|site_args| {
                let (host_cpu_cores, host_memory_in_gb, host_provider) =
                    match site_args.host_provider {
                        HostProviderArg::OciContainerInstance(args) => (
                            args.physics_cpu_cores,
                            args.memory_in_gbs,
                            HostProvider::OciContainerInstance(
                                OciContainerInstanceHostProvider::new(args),
                            ),
                        ),
                    };
                let dns_provider = match site_args.dns_provider {
                    DnsProviderArg::Cloudflare(args) => {
                        DnsProvider::Cloudflare(CloudflareDnsProvider::new(args, None))
                    }
                };

                Site::new(
                    host_provider,
                    dns_provider,
                    args.cert.clone(),
                    deployment_cache.clone(),
                    host_cpu_cores,
                    host_memory_in_gb,
                    doc_db.clone(),
                )
            })
            .collect();

        Ok(HqArgsParsed {
            sites,
            deployment_cache,
        })
    }
}
