pub mod cloudflare;

use std::collections::BTreeSet;

pub trait DnsProvide: Send + Sync {
    async fn sync_addrs(&self, addrs: BTreeSet<String>) -> color_eyre::Result<()>;
}

pub enum DnsProvider {
    Cloudflare(cloudflare::CloudflareDnsProvider),
}

impl DnsProvide for DnsProvider {
    async fn sync_addrs(&self, addrs: BTreeSet<String>) -> color_eyre::Result<()> {
        match self {
            DnsProvider::Cloudflare(cloudflare) => cloudflare.sync_addrs(addrs).await,
        }
    }
}
