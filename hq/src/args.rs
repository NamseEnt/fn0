#![allow(dead_code)]

use std::{collections::BTreeMap, num::NonZeroUsize};

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HqArgs {
    pub sites: Vec<SiteArgs>,
    pub doc_db: DocDbArgs,
    pub cert: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SiteArgs {
    pub host_provider: HostProviderArg,
    pub dns_provider: DnsProviderArg,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum HostProviderArg {
    OciContainerInstance(OciContainerInstanceHostProviderArgs),
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OciContainerInstanceHostProviderArgs {
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
    pub image: String,
    pub envs: BTreeMap<String, String>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum DnsProviderArg {
    Cloudflare(CloudflareDnsProviderArgs),
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
