use std::{collections::BTreeMap, num::NonZeroUsize};

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HqArgs {
    pub sites: Vec<SiteArgs>,
    pub doc_db: DocDbArgs,
    pub forte_db: ForteDbArgs,
    pub forte_r2: ForteR2Args,
    pub aws: AwsArgs,
    pub cwasm_bucket: CwasmBucketArgs,
    pub self_dns: SelfDnsArgs,
    pub dns_provider: DnsProviderArg,
    pub cloudflare_saas: CloudflareSaasArgs,
    pub env_encryption_key_base64: String,
    pub admin_signing_key_base64: String,
}

#[derive(Clone, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CloudflareSaasArgs {
    pub zone_id: String,
    pub api_token: String,
}

#[derive(Clone, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ForteR2Args {
    pub bucket: String,
    pub endpoint: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub public_base_url: String,
}

#[derive(Clone, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ForteDbArgs {
    pub api_token: String,
    pub organization_slug: String,
    pub group_name: String,
}

#[derive(Clone, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelfDnsArgs {
    pub hostname: String,
    pub cloudflare_zone_id: String,
    pub cloudflare_api_token: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AwsArgs {
    pub region: String,
    pub wasm_bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CwasmBucketArgs {
    pub name: String,
    pub endpoint: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SiteArgs {
    pub name: String,
    pub host_provider: HostProviderArg,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum HostProviderArg {
    OciComputeVm(OciComputeVmHostProviderArgs),
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum DnsProviderArg {
    Cloudflare(CloudflareDnsProviderArgs),
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OciComputeVmHostProviderArgs {
    pub private_key_base64: String,
    pub user_id: String,
    pub fingerprint: String,
    pub tenancy_id: String,
    pub region: String,
    pub compartment_id: String,
    pub availability_domain: String,
    pub shape: String,
    pub ocpus: NonZeroUsize,
    pub physics_cpu_cores: NonZeroUsize,
    pub memory_in_gbs: NonZeroUsize,
    pub subnet_id: String,
    pub image_id: String,
    pub envs: BTreeMap<String, String>,
    pub ssh_authorized_keys: String,
    pub ssh_private_key_base64: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CloudflareDnsProviderArgs {
    pub zone_id: String,
    pub asterisk_domain: String,
    pub api_token: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DocDbArgs {
    pub url: String,
    pub token: String,
}
