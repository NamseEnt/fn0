pub mod cloudflare;

use std::{net::IpAddr, pin::Pin};

pub trait Dns: Send + Sync {
    fn sync_ips<'a>(
        &'a self,
        ips: Vec<IpAddr>,
    ) -> Pin<Box<dyn Future<Output = color_eyre::Result<()>> + 'a + Send>>;
}
