pub mod cloudflare;

use std::{collections::BTreeSet, net::IpAddr};

pub trait DnsProvide: Send + Sync {
    async fn sync_ips(&self, ips: BTreeSet<IpAddr>) -> color_eyre::Result<()>;
}

pub enum DnsProvider {
    Cloudflare(cloudflare::CloudflareDnsProvider),
}

impl DnsProvide for DnsProvider {
    async fn sync_ips(&self, ips: BTreeSet<IpAddr>) -> color_eyre::Result<()> {
        match self {
            DnsProvider::Cloudflare(cloudflare) => cloudflare.sync_ips(ips).await,
        }
    }
}
