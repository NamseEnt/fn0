use std::sync::Arc;

use color_eyre::eyre::{Result, eyre};
use opendal::Operator;
use opendal::services::S3;

use crate::doc_db::DocDb;
use crate::lambda::LambdaClient;

use crate::{
    args::*,
    deployment_cache::DeploymentCache,
    dns::{DnsProvider, cloudflare::CloudflareDnsProvider},
    forte_r2::ForteR2,
    host_provider,
    site::Site,
};

pub struct DeployContext {
    pub aws_s3: Operator,
    pub cwasm_s3: Operator,
    pub lambda_client: LambdaClient,
    pub wasm_bucket: String,
    pub cwasm_bucket: String,
    pub doc_db: DocDb,
    pub deployment_cache: DeploymentCache,
    pub sites: Vec<Site>,
    pub env_encryption_key: [u8; 32],
    pub forte_db: crate::args::ForteDbArgs,
    pub forte_r2: ForteR2,
}

pub struct HqArgsParsed {
    pub sites: Vec<Site>,
    pub deployment_cache: DeploymentCache,
    pub deploy_context: Arc<DeployContext>,
    pub self_dns_args: crate::args::SelfDnsArgs,
    pub dns_provider: DnsProvider,
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

        let aws_s3 = Operator::new(
            S3::default()
                .bucket(&args.aws.wasm_bucket)
                .region(&args.aws.region)
                .access_key_id(&args.aws.access_key_id)
                .secret_access_key(&args.aws.secret_access_key)
                .enable_virtual_host_style()
                .disable_config_load()
                .disable_ec2_metadata(),
        )?
        .finish();

        let lambda_client = LambdaClient::new(
            &args.aws.region,
            &args.aws.access_key_id,
            &args.aws.secret_access_key,
        );

        let cwasm_s3 = Operator::new(
            S3::default()
                .bucket(&args.cwasm_bucket.name)
                .region(&args.cwasm_bucket.region)
                .endpoint(&args.cwasm_bucket.endpoint)
                .access_key_id(&args.cwasm_bucket.access_key_id)
                .secret_access_key(&args.cwasm_bucket.secret_access_key)
                .disable_config_load()
                .disable_ec2_metadata(),
        )?
        .finish();

        let sites: Vec<Site> = args
            .sites
            .into_iter()
            .map(|site_args| {
                let (host_cpu_cores, host_memory_in_gb, host_provider) =
                    match site_args.host_provider {
                        HostProviderArg::OciComputeVm(args) => (
                            args.physics_cpu_cores,
                            args.memory_in_gbs,
                            host_provider::oci_compute_vm::OciComputeVmHostProvider::new(args),
                        ),
                    };
                Site::new(
                    site_args.name,
                    host_provider,
                    deployment_cache.clone(),
                    host_cpu_cores,
                    host_memory_in_gb,
                    doc_db.clone(),
                )
            })
            .collect();

        let env_encryption_key =
            crate::env_crypto::decode_key_base64(&args.env_encryption_key_base64)?;

        let forte_r2 = ForteR2::new(&args.forte_r2).await?;

        let deploy_context = Arc::new(DeployContext {
            aws_s3,
            cwasm_s3,
            lambda_client,
            wasm_bucket: args.aws.wasm_bucket,
            cwasm_bucket: args.cwasm_bucket.name,
            doc_db: doc_db.clone(),
            deployment_cache: deployment_cache.clone(),
            sites: sites.clone(),
            env_encryption_key,
            forte_db: args.forte_db,
            forte_r2,
        });

        let dns_provider = match args.dns_provider {
            DnsProviderArg::Cloudflare(dns_args) => {
                DnsProvider::Cloudflare(CloudflareDnsProvider::new(dns_args, None))
            }
        };

        Ok(HqArgsParsed {
            sites,
            deployment_cache,
            deploy_context,
            self_dns_args: args.self_dns.clone(),
            dns_provider,
        })
    }
}
