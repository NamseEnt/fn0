//! Destination policy for connections that application code asks the host to open.
//!
//! Guest HTTP requests, JavaScript `fetch`, and outbound WebSockets all name their destination
//! by URL, but the socket is opened by the worker process, which can reach addresses the public
//! internet cannot: loopback services, the private network, and cloud metadata endpoints.
//! [`OutboundDialer`] resolves the host once, removes every non-public address, and connects to
//! the exact [`SocketAddr`] it approved, so a DNS answer that changes between the check and the
//! connection cannot redirect the socket. Every dial resolves again, so a later reconnect is
//! checked against the answer that is current at that time.

use std::future::Future;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;

pub const ALLOW_PRIVATE_OUTBOUND_DESTINATIONS_ENV: &str = "FN0_ALLOW_PRIVATE_OUTBOUND_DESTINATIONS";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrivateDestinationAccess {
    Blocked,
    Allowed,
}

impl PrivateDestinationAccess {
    /// Reads [`ALLOW_PRIVATE_OUTBOUND_DESTINATIONS_ENV`]. Unset means blocked; any value other
    /// than `true` or `false` panics so a typo cannot silently open the private network.
    pub fn from_env() -> Self {
        match std::env::var(ALLOW_PRIVATE_OUTBOUND_DESTINATIONS_ENV) {
            Err(std::env::VarError::NotPresent) => Self::Blocked,
            Ok(value) if value == "false" => Self::Blocked,
            Ok(value) if value == "true" => Self::Allowed,
            Ok(value) => panic!(
                "{ALLOW_PRIVATE_OUTBOUND_DESTINATIONS_ENV} must be `true` or `false`, got `{value}`"
            ),
            Err(std::env::VarError::NotUnicode(_)) => {
                panic!("{ALLOW_PRIVATE_OUTBOUND_DESTINATIONS_ENV} must be `true` or `false`")
            }
        }
    }

    pub fn permits(self, address: IpAddr) -> bool {
        match self {
            Self::Allowed => true,
            Self::Blocked => is_public_internet_address(address),
        }
    }
}

pub type ResolvedAddressesFuture =
    Pin<Box<dyn Future<Output = std::io::Result<Vec<SocketAddr>>> + Send + 'static>>;

pub trait DestinationResolver: Send + Sync {
    fn resolve(&self, host: &str, port: u16) -> ResolvedAddressesFuture;
}

pub struct SystemDestinationResolver;

impl DestinationResolver for SystemDestinationResolver {
    fn resolve(&self, host: &str, port: u16) -> ResolvedAddressesFuture {
        let host = host.to_string();
        Box::pin(async move {
            Ok(tokio::net::lookup_host((host.as_str(), port))
                .await?
                .collect())
        })
    }
}

#[derive(Debug)]
pub enum OutboundDialError {
    DestinationForbidden,
    NameResolution(std::io::Error),
    NoAddresses,
    Connect(std::io::Error),
    Timeout,
}

impl std::fmt::Display for OutboundDialError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DestinationForbidden => {
                formatter.write_str("outbound destination resolves only to non-public addresses")
            }
            Self::NameResolution(error) => write!(formatter, "name resolution failed: {error}"),
            Self::NoAddresses => formatter.write_str("name resolved to no addresses"),
            Self::Connect(error) => write!(formatter, "connect failed: {error}"),
            Self::Timeout => formatter.write_str("connect timed out"),
        }
    }
}

impl std::error::Error for OutboundDialError {}

#[derive(Clone)]
pub struct OutboundDialer {
    resolver: Arc<dyn DestinationResolver>,
    private_destination_access: PrivateDestinationAccess,
}

impl OutboundDialer {
    pub fn new(
        resolver: Arc<dyn DestinationResolver>,
        private_destination_access: PrivateDestinationAccess,
    ) -> Self {
        Self {
            resolver,
            private_destination_access,
        }
    }

    pub fn system(private_destination_access: PrivateDestinationAccess) -> Self {
        Self::new(
            Arc::new(SystemDestinationResolver),
            private_destination_access,
        )
    }

    pub fn with_private_destination_access(
        &self,
        private_destination_access: PrivateDestinationAccess,
    ) -> Self {
        Self::new(self.resolver.clone(), private_destination_access)
    }

    pub fn private_destination_access(&self) -> PrivateDestinationAccess {
        self.private_destination_access
    }

    pub async fn permitted_addresses(
        &self,
        host: &str,
        port: u16,
    ) -> Result<Vec<SocketAddr>, OutboundDialError> {
        let unbracketed_host = host
            .strip_prefix('[')
            .and_then(|inner| inner.strip_suffix(']'))
            .unwrap_or(host);
        let resolved_addresses = match unbracketed_host.parse::<IpAddr>() {
            Ok(literal_address) => vec![SocketAddr::new(literal_address, port)],
            Err(_) => self
                .resolver
                .resolve(unbracketed_host, port)
                .await
                .map_err(OutboundDialError::NameResolution)?,
        };
        if resolved_addresses.is_empty() {
            return Err(OutboundDialError::NoAddresses);
        }
        let permitted_addresses: Vec<SocketAddr> = resolved_addresses
            .into_iter()
            .filter(|address| self.private_destination_access.permits(address.ip()))
            .collect();
        if permitted_addresses.is_empty() {
            return Err(OutboundDialError::DestinationForbidden);
        }
        Ok(permitted_addresses)
    }

    pub async fn connect(
        &self,
        host: &str,
        port: u16,
        connect_timeout: Duration,
    ) -> Result<TcpStream, OutboundDialError> {
        let permitted_addresses =
            tokio::time::timeout(connect_timeout, self.permitted_addresses(host, port))
                .await
                .map_err(|_| OutboundDialError::Timeout)??;
        let deadline = tokio::time::Instant::now() + connect_timeout;
        let mut last_error = None;
        for address in permitted_addresses {
            match tokio::time::timeout_at(deadline, TcpStream::connect(address)).await {
                Ok(Ok(stream)) => return Ok(stream),
                Ok(Err(error)) => last_error = Some(error),
                Err(_) => return Err(OutboundDialError::Timeout),
            }
        }
        Err(last_error.map_or(OutboundDialError::NoAddresses, OutboundDialError::Connect))
    }
}

pub fn is_public_internet_address(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => is_public_ipv4(address),
        IpAddr::V6(address) => is_public_ipv6(address),
    }
}

fn is_public_ipv4(address: Ipv4Addr) -> bool {
    let [first, second, third, _] = address.octets();
    let in_shared_address_space = first == 100 && (64..=127).contains(&second);
    let in_protocol_assignments = first == 192 && second == 0 && third == 0;
    let in_benchmarking = first == 198 && (second == 18 || second == 19);
    let in_reserved = first >= 240;
    !(first == 0
        || address.is_private()
        || address.is_loopback()
        || address.is_link_local()
        || address.is_multicast()
        || address.is_broadcast()
        || address.is_documentation()
        || in_shared_address_space
        || in_protocol_assignments
        || in_benchmarking
        || in_reserved)
}

fn is_public_ipv6(address: Ipv6Addr) -> bool {
    let segments = address.segments();
    if segments[..5] == [0, 0, 0, 0, 0] {
        return segments[5] == 0xffff
            && is_public_ipv4(Ipv4Addr::new(
                (segments[6] >> 8) as u8,
                segments[6] as u8,
                (segments[7] >> 8) as u8,
                segments[7] as u8,
            ));
    }
    if segments[0] == 0x0064 && segments[1] == 0xff9b {
        return segments[2..6] == [0, 0, 0, 0]
            && is_public_ipv4(Ipv4Addr::new(
                (segments[6] >> 8) as u8,
                segments[6] as u8,
                (segments[7] >> 8) as u8,
                segments[7] as u8,
            ));
    }
    if segments[0] == 0x2002 {
        return is_public_ipv4(Ipv4Addr::new(
            (segments[1] >> 8) as u8,
            segments[1] as u8,
            (segments[2] >> 8) as u8,
            segments[2] as u8,
        ));
    }
    let discard_only = segments[..4] == [0x0100, 0, 0, 0];
    let teredo = segments[0] == 0x2001 && segments[1] == 0;
    let documentation = segments[0] == 0x2001 && segments[1] == 0x0db8;
    let site_local = (segments[0] & 0xffc0) == 0xfec0;
    !(address.is_multicast()
        || address.is_unique_local()
        || address.is_unicast_link_local()
        || discard_only
        || teredo
        || documentation
        || site_local)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct ScriptedResolver {
        answers: Mutex<Vec<Vec<IpAddr>>>,
        resolved_hosts: Mutex<Vec<String>>,
    }

    impl ScriptedResolver {
        fn new(answers: Vec<Vec<IpAddr>>) -> Self {
            Self {
                answers: Mutex::new(answers),
                resolved_hosts: Mutex::new(Vec::new()),
            }
        }
    }

    impl DestinationResolver for ScriptedResolver {
        fn resolve(&self, host: &str, port: u16) -> ResolvedAddressesFuture {
            self.resolved_hosts.lock().unwrap().push(host.to_string());
            let answer = self.answers.lock().unwrap().remove(0);
            Box::pin(async move {
                Ok(answer
                    .into_iter()
                    .map(|address| SocketAddr::new(address, port))
                    .collect())
            })
        }
    }

    fn address(text: &str) -> IpAddr {
        text.parse().unwrap()
    }

    #[test]
    fn non_public_ipv4_addresses_are_rejected() {
        for text in [
            "0.0.0.0",
            "0.1.2.3",
            "10.0.0.1",
            "100.64.0.1",
            "100.127.255.254",
            "127.0.0.1",
            "169.254.169.254",
            "172.16.0.1",
            "172.31.255.255",
            "192.0.0.8",
            "192.0.2.1",
            "192.168.0.10",
            "198.18.0.1",
            "198.51.100.1",
            "203.0.113.1",
            "224.0.0.1",
            "240.0.0.1",
            "255.255.255.255",
        ] {
            assert!(!is_public_internet_address(address(text)), "{text}");
        }
    }

    #[test]
    fn non_public_ipv6_addresses_are_rejected() {
        for text in [
            "::",
            "::1",
            "::127.0.0.1",
            "::ffff:127.0.0.1",
            "::ffff:169.254.169.254",
            "64:ff9b::a9fe:a9fe",
            "64:ff9b:1::1",
            "100::1",
            "2001::1",
            "2001:db8::1",
            "2002:c0a8:000a::1",
            "fc00::1",
            "fd12:3456::1",
            "fe80::1",
            "fec0::1",
            "ff02::1",
        ] {
            assert!(!is_public_internet_address(address(text)), "{text}");
        }
    }

    #[test]
    fn public_addresses_are_accepted() {
        for text in [
            "1.1.1.1",
            "8.8.8.8",
            "100.63.255.255",
            "100.128.0.1",
            "172.32.0.1",
            "2606:4700:4700::1111",
            "::ffff:8.8.8.8",
            "64:ff9b::808:808",
            "2002:0808:0808::1",
        ] {
            assert!(is_public_internet_address(address(text)), "{text}");
        }
    }

    #[test]
    fn allowed_access_permits_private_addresses() {
        assert!(PrivateDestinationAccess::Allowed.permits(address("127.0.0.1")));
        assert!(!PrivateDestinationAccess::Blocked.permits(address("127.0.0.1")));
    }

    #[tokio::test]
    async fn literal_private_host_is_rejected_without_resolution() {
        let resolver = Arc::new(ScriptedResolver::new(Vec::new()));
        let dialer = OutboundDialer::new(resolver.clone(), PrivateDestinationAccess::Blocked);
        for host in ["127.0.0.1", "[::1]", "::ffff:10.0.0.1"] {
            assert!(matches!(
                dialer.permitted_addresses(host, 443).await,
                Err(OutboundDialError::DestinationForbidden)
            ));
        }
        assert!(resolver.resolved_hosts.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn mixed_answer_keeps_only_public_addresses_with_requested_port() {
        let resolver = Arc::new(ScriptedResolver::new(vec![vec![
            address("10.0.0.5"),
            address("93.184.216.34"),
            address("::1"),
            address("2606:2800:220:1:248:1893:25c8:1946"),
        ]]));
        let dialer = OutboundDialer::new(resolver, PrivateDestinationAccess::Blocked);
        let permitted = dialer
            .permitted_addresses("example.com", 8443)
            .await
            .unwrap();
        assert_eq!(
            permitted,
            vec![
                SocketAddr::new(address("93.184.216.34"), 8443),
                SocketAddr::new(address("2606:2800:220:1:248:1893:25c8:1946"), 8443),
            ]
        );
    }

    #[tokio::test]
    async fn rebinding_to_private_address_is_rejected_on_the_next_dial() {
        let resolver = Arc::new(ScriptedResolver::new(vec![
            vec![address("93.184.216.34")],
            vec![address("169.254.169.254")],
        ]));
        let dialer = OutboundDialer::new(resolver.clone(), PrivateDestinationAccess::Blocked);
        assert!(
            dialer
                .permitted_addresses("rebind.example", 443)
                .await
                .is_ok()
        );
        assert!(matches!(
            dialer.permitted_addresses("rebind.example", 443).await,
            Err(OutboundDialError::DestinationForbidden)
        ));
        assert_eq!(
            resolver.resolved_hosts.lock().unwrap().as_slice(),
            ["rebind.example", "rebind.example"]
        );
    }

    #[tokio::test]
    async fn connect_uses_the_approved_address_instead_of_resolving_again() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let listener_port = listener.local_addr().unwrap().port();
        let resolver = Arc::new(ScriptedResolver::new(vec![vec![address("127.0.0.1")]]));
        let dialer = OutboundDialer::new(resolver.clone(), PrivateDestinationAccess::Allowed);
        let accept_task = tokio::spawn(async move { listener.accept().await.unwrap() });
        let stream = dialer
            .connect("service.internal", listener_port, Duration::from_secs(1))
            .await
            .unwrap();
        assert_eq!(stream.peer_addr().unwrap().port(), listener_port);
        accept_task.await.unwrap();
        assert_eq!(resolver.resolved_hosts.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn blocked_connect_never_opens_a_socket_to_loopback() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let listener_port = listener.local_addr().unwrap().port();
        let resolver = Arc::new(ScriptedResolver::new(vec![vec![address("127.0.0.1")]]));
        let dialer = OutboundDialer::new(resolver, PrivateDestinationAccess::Blocked);
        assert!(matches!(
            dialer
                .connect("localhost.example", listener_port, Duration::from_secs(1))
                .await,
            Err(OutboundDialError::DestinationForbidden)
        ));
        assert!(
            tokio::time::timeout(Duration::from_millis(50), listener.accept())
                .await
                .is_err()
        );
    }
}
