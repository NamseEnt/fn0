//! A minimal HTTP/1.1 client over tokio, with keep-alive.
//!
//! Hand-rolled rather than pulled in, because a dependency added for the
//! harness is a dependency the server binary links: `object_store` is the only
//! outbound HTTP client in this process and `reqwest` was removed to keep it
//! that way (`Cargo.toml`). What this needs is one connection held open across
//! many requests, which is a few hundred lines.
//!
//! The connection this replaces opened a TCP socket per request and sent
//! `Connection: close` (`bin/load.rs:715,732` before the rewrite), so every
//! latency it reported included a handshake and keep-alive was never
//! exercised at all.

use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

pub struct Request<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub body: &'a [u8],
    pub content_type: &'a str,
    /// The tenant header to send, as `(name, value)`. `None` on a signy write:
    /// the tenant is inside the body there.
    pub tenant: Option<(&'a str, &'a str)>,
    /// Anything else the endpoint needs, as `(name, value)`. signy's collect
    /// route wants the batch's compression and which signal it carries; no
    /// other request sends any.
    pub headers: &'a [(&'a str, &'a str)],
}

pub struct Response {
    pub status: u16,
    pub body: Vec<u8>,
}

pub struct Conn {
    address: String,
    stream: BufReader<TcpStream>,
    timeout: Duration,
    /// False once the peer has said, or implied, that this connection ends
    /// with the current response. The caller reconnects rather than writing
    /// another request into a socket that is closing.
    reusable: bool,
}

impl Conn {
    pub async fn connect(address: &str, timeout: Duration) -> Result<Self, String> {
        let stream = tokio::time::timeout(timeout, TcpStream::connect(address))
            .await
            .map_err(|_| format!("connecting to {address} timed out"))?
            .map_err(|error| error.to_string())?;
        // Latency is the measurement here, and Nagle interacts with delayed
        // ACK to add tens of milliseconds to a request whose bytes leave in
        // more than one write.
        stream
            .set_nodelay(true)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            address: address.to_string(),
            stream: BufReader::new(stream),
            timeout,
            reusable: true,
        })
    }

    pub fn reusable(&self) -> bool {
        self.reusable
    }

    pub async fn request(&mut self, request: &Request<'_>) -> Result<Response, String> {
        match tokio::time::timeout(self.timeout, self.exchange(request)).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(error)) => {
                self.reusable = false;
                Err(error)
            }
            Err(_) => {
                // A timed-out exchange leaves an unread response in the
                // socket, so the connection cannot be reused whatever the
                // server does next.
                self.reusable = false;
                Err(format!("{} {} timed out", request.method, request.path))
            }
        }
    }

    async fn exchange(&mut self, request: &Request<'_>) -> Result<Response, String> {
        let mut wire = Vec::with_capacity(request.body.len() + 256);
        // No `Connection` header: HTTP/1.1 keeps the connection open unless
        // told otherwise, and that default is the point of this client.
        wire.extend_from_slice(
            format!(
                "{} {} HTTP/1.1\r\nHost: {}\r\n",
                request.method, request.path, self.address
            )
            .as_bytes(),
        );
        if let Some((name, value)) = request.tenant {
            wire.extend_from_slice(format!("{name}: {value}\r\n").as_bytes());
        }
        for (name, value) in request.headers {
            wire.extend_from_slice(format!("{name}: {value}\r\n").as_bytes());
        }
        if !request.content_type.is_empty() {
            wire.extend_from_slice(
                format!("Content-Type: {}\r\n", request.content_type).as_bytes(),
            );
        }
        wire.extend_from_slice(
            format!("Content-Length: {}\r\n\r\n", request.body.len()).as_bytes(),
        );
        wire.extend_from_slice(request.body);
        self.stream
            .get_mut()
            .write_all(&wire)
            .await
            .map_err(|error| error.to_string())?;
        self.read_response().await
    }

    async fn read_response(&mut self) -> Result<Response, String> {
        let mut line = String::new();
        if self
            .stream
            .read_line(&mut line)
            .await
            .map_err(|error| error.to_string())?
            == 0
        {
            return Err("connection closed before a response arrived".to_string());
        }
        let status: u16 = line
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| format!("malformed status line: {}", line.trim_end()))?
            .parse()
            .map_err(|error| format!("malformed HTTP status: {error}"))?;

        let mut content_length: Option<usize> = None;
        let mut chunked = false;
        loop {
            line.clear();
            if self
                .stream
                .read_line(&mut line)
                .await
                .map_err(|error| error.to_string())?
                == 0
            {
                return Err("connection closed inside the response headers".to_string());
            }
            let header = line.trim_end();
            if header.is_empty() {
                break;
            }
            let Some((name, value)) = header.split_once(':') else {
                continue;
            };
            let value = value.trim();
            match name.trim().to_ascii_lowercase().as_str() {
                "content-length" => content_length = value.parse().ok(),
                "transfer-encoding" => {
                    chunked |= value.to_ascii_lowercase().contains("chunked");
                }
                "connection" if value.eq_ignore_ascii_case("close") => self.reusable = false,
                _ => {}
            }
        }

        let body = if status == 204 || status == 304 || (100..200).contains(&status) {
            Vec::new()
        } else if chunked {
            self.read_chunked().await?
        } else if let Some(length) = content_length {
            let mut body = vec![0u8; length];
            self.stream
                .read_exact(&mut body)
                .await
                .map_err(|error| error.to_string())?;
            body
        } else {
            // No framing at all means the body ends when the connection does,
            // so this one is spent.
            self.reusable = false;
            let mut body = Vec::new();
            self.stream
                .read_to_end(&mut body)
                .await
                .map_err(|error| error.to_string())?;
            body
        };
        Ok(Response { status, body })
    }

    async fn read_chunked(&mut self) -> Result<Vec<u8>, String> {
        let mut body = Vec::new();
        let mut line = String::new();
        loop {
            line.clear();
            if self
                .stream
                .read_line(&mut line)
                .await
                .map_err(|error| error.to_string())?
                == 0
            {
                return Err("connection closed inside a chunked body".to_string());
            }
            let header = line.trim_end();
            let size_text = header.split(';').next().unwrap_or("").trim();
            let size = usize::from_str_radix(size_text, 16)
                .map_err(|error| format!("malformed chunk size {size_text:?}: {error}"))?;
            if size == 0 {
                loop {
                    line.clear();
                    let read = self
                        .stream
                        .read_line(&mut line)
                        .await
                        .map_err(|error| error.to_string())?;
                    if read == 0 || line.trim_end().is_empty() {
                        break;
                    }
                }
                return Ok(body);
            }
            let start = body.len();
            body.resize(start + size, 0);
            self.stream
                .read_exact(&mut body[start..])
                .await
                .map_err(|error| error.to_string())?;
            let mut terminator = [0u8; 2];
            self.stream
                .read_exact(&mut terminator)
                .await
                .map_err(|error| error.to_string())?;
        }
    }
}

/// A connection that reconnects when the previous request killed it.
///
/// Every workload here is one request in flight per connection, so a fault
/// costs the connection and nothing else; the alternative is a run that
/// reports a hundred percent errors because the first one closed the socket.
pub struct Client {
    address: String,
    timeout: Duration,
    conn: Option<Conn>,
    /// TCP connections this client has opened. With keep-alive working it
    /// stays at one; anything more is a fault, and the count is the evidence.
    pub connects: u64,
}

impl Client {
    pub fn new(address: &str, timeout: Duration) -> Self {
        Self {
            address: address.to_string(),
            timeout,
            conn: None,
            connects: 0,
        }
    }

    pub async fn request(&mut self, request: &Request<'_>) -> Result<Response, String> {
        if self.conn.as_ref().is_none_or(|conn| !conn.reusable()) {
            self.conn = None;
            self.connects += 1;
            self.conn = Some(Conn::connect(&self.address, self.timeout).await?);
        }
        let conn = self.conn.as_mut().expect("connection was just established");
        conn.request(request).await
    }
}
