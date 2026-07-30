//! P0 probe for issue #80: does JSPI-backed async lifting of a `wasi:http` p3
//! component work inside a deployed Cloudflare Worker?
//!
//! One request exercises the three parts of the p3 surface forte-sdk depends on,
//! and nothing else — no forte-sdk, so a failure points at the toolchain:
//!
//! 1. `monotonic-clock.wait-for` — an async import that must suspend the guest
//!    stack and resume it.
//! 2. `http/client.send` — an async import returning a resource, plus reading
//!    its body stream to completion.
//! 3. `http/handler.handle` — an async export whose response body is a
//!    `stream<u8>` fed by a `wit_bindgen::spawn`ed task, which is how
//!    `forte_sdk::serve` builds every response.

wit_bindgen::generate!({
    inline: "
        package probe:jspi;
        world probe {
            import wasi:clocks/types@0.3.0-rc-2026-03-15;
            import wasi:clocks/monotonic-clock@0.3.0-rc-2026-03-15;
            import wasi:http/types@0.3.0-rc-2026-03-15;
            import wasi:http/client@0.3.0-rc-2026-03-15;
            export wasi:http/handler@0.3.0-rc-2026-03-15;
        }
    ",
    path: "wit",
    world: "probe",
    generate_all,
    features: ["clocks-timezone"],
});

use exports::wasi::http::handler::Guest;
use wasi::clocks::monotonic_clock;
use wasi::http::client;
use wasi::http::types as p3;

const WAIT_NANOS: u64 = 5_000_000;
const FETCH_URL: &str = "https://example.com/";

struct Probe;

impl Guest for Probe {
    async fn handle(request: p3::Request) -> Result<p3::Response, p3::ErrorCode> {
        drop(request);

        let before_wait = monotonic_clock::now();
        monotonic_clock::wait_for(WAIT_NANOS).await;
        let after_wait = monotonic_clock::now();

        let before_fetch = monotonic_clock::now();
        let fetched = fetch(FETCH_URL).await?;
        let after_fetch = monotonic_clock::now();

        respond(
            fetched,
            Timings {
                waited_nanos: after_wait.saturating_sub(before_wait),
                fetch_nanos: after_fetch.saturating_sub(before_fetch),
            },
        )
        .await
    }
}

export!(Probe);

struct Timings {
    waited_nanos: u64,
    fetch_nanos: u64,
}

async fn fetch(url: &str) -> Result<Vec<u8>, p3::ErrorCode> {
    let uri = ParsedUri::parse(url).ok_or_else(|| unparseable_url(url))?;

    let fields =
        p3::Fields::from_list(&[("host".to_string(), uri.authority.clone().into_bytes())])
            .map_err(header_error)?;

    let (trailers_writer, trailers_reader) =
        wit_future::new::<Result<Option<p3::Trailers>, p3::ErrorCode>>(|| Ok(None));
    wit_bindgen::spawn(async move {
        drop(trailers_writer);
    });

    let (request, _transmit) = p3::Request::new(fields, None, trailers_reader, None);
    request
        .set_method(&p3::Method::Get)
        .map_err(|()| internal("set_method rejected GET"))?;
    request
        .set_scheme(Some(&uri.scheme))
        .map_err(|()| internal("set_scheme rejected the parsed scheme"))?;
    request
        .set_authority(Some(&uri.authority))
        .map_err(|()| internal("set_authority rejected the parsed authority"))?;
    request
        .set_path_with_query(Some(&uri.path_with_query))
        .map_err(|()| internal("set_path_with_query rejected the parsed path"))?;

    let response = client::send(request).await?;
    let status = response.get_status_code();
    if status != 200 {
        return Err(internal(&format!("upstream returned {status}")));
    }

    let (body_trailers_writer, body_trailers_reader) =
        wit_future::new::<Result<(), p3::ErrorCode>>(|| Ok(()));
    wit_bindgen::spawn(async move {
        drop(body_trailers_writer);
    });
    let (body, _trailers) = p3::Response::consume_body(response, body_trailers_reader);

    Ok(body.collect().await)
}

async fn respond(body: Vec<u8>, timings: Timings) -> Result<p3::Response, p3::ErrorCode> {
    let byte_count = body.len();
    let fields = p3::Fields::from_list(&[
        ("content-type".to_string(), b"text/html".to_vec()),
        (
            "x-probe-waited-nanos".to_string(),
            timings.waited_nanos.to_string().into_bytes(),
        ),
        (
            "x-probe-fetch-nanos".to_string(),
            timings.fetch_nanos.to_string().into_bytes(),
        ),
        (
            "x-probe-fetched-bytes".to_string(),
            byte_count.to_string().into_bytes(),
        ),
    ])
    .map_err(header_error)?;

    let (mut writer, reader) = wit_stream::new::<u8>();
    wit_bindgen::spawn(async move {
        let _leftover = writer.write_all(body).await;
        drop(writer);
    });

    let (trailers_writer, trailers_reader) =
        wit_future::new::<Result<Option<p3::Trailers>, p3::ErrorCode>>(|| Ok(None));
    wit_bindgen::spawn(async move {
        drop(trailers_writer);
    });

    let (response, _transmit) = p3::Response::new(fields, Some(reader), trailers_reader);
    response
        .set_status_code(200)
        .map_err(|()| internal("set_status_code rejected 200"))?;

    Ok(response)
}

struct ParsedUri {
    scheme: p3::Scheme,
    authority: String,
    path_with_query: String,
}

impl ParsedUri {
    fn parse(url: &str) -> Option<Self> {
        let (scheme, rest) = match url.split_once("://")? {
            ("http", rest) => (p3::Scheme::Http, rest),
            ("https", rest) => (p3::Scheme::Https, rest),
            (other, rest) => (p3::Scheme::Other(other.to_string()), rest),
        };
        let (authority, path_with_query) = match rest.find('/') {
            Some(slash) => (rest[..slash].to_string(), rest[slash..].to_string()),
            None => (rest.to_string(), "/".to_string()),
        };
        if authority.is_empty() {
            return None;
        }
        Some(Self {
            scheme,
            authority,
            path_with_query,
        })
    }
}

fn internal(message: &str) -> p3::ErrorCode {
    p3::ErrorCode::InternalError(Some(message.to_string()))
}

fn header_error(error: p3::HeaderError) -> p3::ErrorCode {
    internal(&format!("header rejected: {error:?}"))
}

fn unparseable_url(url: &str) -> p3::ErrorCode {
    internal(&format!("could not parse {url}"))
}
