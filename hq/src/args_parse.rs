use std::sync::Arc;

use aws_sdk_s3::Client as S3Client;
use color_eyre::eyre::{Result, eyre};
use doc_db::DocDb;

use crate::{
    args::*,
    deployment_cache::DeploymentCache,
    dns::{DnsProvider, cloudflare::CloudflareDnsProvider},
    host_provider::{self, HostProvider},
    site::Site,
};

pub struct DeployContext {
    pub s3_client: S3Client,
    pub wasm_bucket: String,
    pub doc_db: DocDb,
}

pub struct HqArgsParsed {
    pub sites: Vec<Site>,
    pub deployment_cache: DeploymentCache,
    pub deploy_context: Arc<DeployContext>,
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

        let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(args.aws.region))
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                &args.aws.access_key_id,
                &args.aws.secret_access_key,
                None,
                None,
                "hq-config",
            ))
            .load()
            .await;
        let s3_client = S3Client::new(&aws_config);

        let deploy_context = Arc::new(DeployContext {
            s3_client,
            wasm_bucket: args.aws.wasm_bucket,
            doc_db: doc_db.clone(),
        });

        let sites = args
            .sites
            .into_iter()
            .map(|site_args| {
                let (host_cpu_cores, host_memory_in_gb, host_provider) =
                    match site_args.host_provider {
                        HostProviderArg::OciComputeVm(args) => (
                            Some(args.physics_cpu_cores),
                            Some(args.memory_in_gbs),
                            HostProvider::OciComputeVm(
                                host_provider::oci_compute_vm::OciComputeVmHostProvider::new(args),
                            ),
                        ),
                        HostProviderArg::Dedicated(_) => (
                            None,
                            None,
                            HostProvider::Dedicated(
                                host_provider::dedicated::DedicatedHostProvider::new(
                                    doc_db.clone(),
                                ),
                            ),
                        ),
                    };
                let dns_provider = match site_args.dns_provider {
                    DnsProviderArg::Cloudflare(args) => {
                        DnsProvider::Cloudflare(CloudflareDnsProvider::new(args, None))
                    }
                };

                Site::new(
                    site_args.name,
                    host_provider,
                    dns_provider,
                    args.ca_cert_pem.clone(),
                    deployment_cache.clone(),
                    host_cpu_cores,
                    host_memory_in_gb,
                    doc_db.clone(),
                    args.ws_secret.clone(),
                )
            })
            .collect();

        Ok(HqArgsParsed {
            sites,
            deployment_cache,
            deploy_context,
        })
    }
}
