use forte_sdk::*;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

fn sha256_hex(data: &[u8]) -> String {
    forte_sdk::hex::encode(Sha256::digest(data))
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn signing_key(secret: &str, date: &str, region: &str, service: &str) -> Vec<u8> {
    let k_date = hmac_sha256(format!("AWS4{secret}").as_bytes(), date.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, service.as_bytes());
    hmac_sha256(&k_service, b"aws4_request")
}

fn uri_encode(s: &str, encode_slash: bool) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        let unreserved = b.is_ascii_alphanumeric()
            || b == b'-'
            || b == b'_'
            || b == b'.'
            || b == b'~'
            || (b == b'/' && !encode_slash);
        if unreserved {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

pub struct R2PresignArgs<'a> {
    pub account_id: &'a str,
    pub bucket: &'a str,
    pub region: &'a str,
    pub key: &'a str,
    pub access_key_id: &'a str,
    pub secret_access_key: &'a str,
    pub expires_seconds: u32,
    pub now: DateTime,
}

pub fn r2_presign_put(args: R2PresignArgs<'_>) -> String {
    let host = format!(
        "{}.{}.r2.cloudflarestorage.com",
        args.bucket, args.account_id
    );
    let amz_date = args.now.format("%Y%m%dT%H%M%SZ").to_string();
    let date = args.now.format("%Y%m%d").to_string();
    let credential_scope = format!("{date}/{}/s3/aws4_request", args.region);
    let credential = format!("{}/{credential_scope}", args.access_key_id);
    let canonical_uri = format!("/{}", uri_encode(args.key, false));

    let mut query: Vec<(String, String)> = vec![
        ("X-Amz-Algorithm".into(), "AWS4-HMAC-SHA256".into()),
        ("X-Amz-Credential".into(), credential),
        ("X-Amz-Date".into(), amz_date.clone()),
        ("X-Amz-Expires".into(), args.expires_seconds.to_string()),
        ("X-Amz-SignedHeaders".into(), "host".into()),
    ];
    query.sort_by(|a, b| a.0.cmp(&b.0));
    let canonical_query = query
        .iter()
        .map(|(k, v)| format!("{}={}", uri_encode(k, true), uri_encode(v, true)))
        .collect::<Vec<_>>()
        .join("&");

    let canonical_headers = format!("host:{host}\n");
    let signed_headers = "host";
    let payload_hash = "UNSIGNED-PAYLOAD";
    let canonical_request = format!(
        "PUT\n{canonical_uri}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}",
    );

    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
        sha256_hex(canonical_request.as_bytes()),
    );
    let key = signing_key(args.secret_access_key, &date, args.region, "s3");
    let signature = forte_sdk::hex::encode(hmac_sha256(&key, string_to_sign.as_bytes()));

    format!("https://{host}{canonical_uri}?{canonical_query}&X-Amz-Signature={signature}")
}

pub struct LambdaInvokeArgs<'a> {
    pub region: &'a str,
    pub access_key_id: &'a str,
    pub secret_access_key: &'a str,
    pub function_name: &'a str,
    pub payload: Vec<u8>,
    pub now: DateTime,
}

pub async fn lambda_invoke_async(args: LambdaInvokeArgs<'_>) -> anyhow::Result<()> {
    let host = format!("lambda.{}.amazonaws.com", args.region);
    let path = format!(
        "/2015-03-31/functions/{}/invocations",
        uri_encode(args.function_name, true),
    );
    let amz_date = args.now.format("%Y%m%dT%H%M%SZ").to_string();
    let date = args.now.format("%Y%m%d").to_string();
    let payload_hash = sha256_hex(&args.payload);
    let credential_scope = format!("{date}/{}/lambda/aws4_request", args.region);
    let credential = format!("{}/{credential_scope}", args.access_key_id);

    let canonical_headers = format!(
        "host:{host}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\nx-amz-invocation-type:Event\n",
    );
    let signed_headers = "host;x-amz-content-sha256;x-amz-date;x-amz-invocation-type";
    let canonical_request = format!(
        "POST\n{path}\n\n{canonical_headers}\n{signed_headers}\n{payload_hash}",
    );
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
        sha256_hex(canonical_request.as_bytes()),
    );
    let key = signing_key(args.secret_access_key, &date, args.region, "lambda");
    let signature = forte_sdk::hex::encode(hmac_sha256(&key, string_to_sign.as_bytes()));
    let auth = format!(
        "AWS4-HMAC-SHA256 Credential={credential}, SignedHeaders={signed_headers}, Signature={signature}",
    );

    let url = format!("https://{host}{path}");
    let request = http::Request::post(&url)
        .header("Authorization", auth)
        .header("x-amz-content-sha256", payload_hash)
        .header("x-amz-date", amz_date)
        .header("x-amz-invocation-type", "Event")
        .header("content-type", "application/json")
        .body(args.payload)?;

    let resp = http::Client::new().send(request).await?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.into_body().bytes().await;
        anyhow::bail!(
            "lambda async invoke {}: {}",
            status,
            String::from_utf8_lossy(&body)
        );
    }
    Ok(())
}

