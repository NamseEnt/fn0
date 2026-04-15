use aws_sdk_lambda::Client as LambdaClient;
use aws_sdk_s3::Client as S3Client;
use aws_smithy_types::Blob;
use color_eyre::eyre::{Result, eyre};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct InvokePayload<'a> {
    input_bucket: &'a str,
    input_key: &'a str,
    output_bucket: &'a str,
    output_key: &'a str,
}

#[derive(Deserialize, Debug)]
struct InvokeResult {
    #[allow(dead_code)]
    output_bucket: String,
    #[allow(dead_code)]
    output_key: String,
    #[allow(dead_code)]
    size: u64,
}

pub fn function_name(fn0_wasmtime_version: &str) -> String {
    format!(
        "fn0-cwasm-compiler-{}",
        fn0_wasmtime_version.replace('.', "-")
    )
}

pub fn raw_key(subdomain: &str) -> String {
    format!("bundles/{subdomain}.raw.tar")
}

pub fn intermediate_key(fn0_wasmtime_version: &str, subdomain: &str) -> String {
    format!("compiled/{fn0_wasmtime_version}/{subdomain}.tar.zst")
}

pub fn final_key(fn0_wasmtime_version: &str, subdomain: &str) -> String {
    format!("bundles/{fn0_wasmtime_version}/{subdomain}.tar.zst")
}

pub async fn compile_and_publish(
    lambda_client: &LambdaClient,
    aws_s3_client: &S3Client,
    cwasm_s3_client: &S3Client,
    wasm_bucket: &str,
    cwasm_bucket: &str,
    fn0_wasmtime_version: &str,
    subdomain: &str,
) -> Result<()> {
    let input_key = raw_key(subdomain);
    let intermediate_key = intermediate_key(fn0_wasmtime_version, subdomain);

    invoke_compile(
        lambda_client,
        fn0_wasmtime_version,
        wasm_bucket,
        &input_key,
        wasm_bucket,
        &intermediate_key,
    )
    .await?;

    let object = aws_s3_client
        .get_object()
        .bucket(wasm_bucket)
        .key(&intermediate_key)
        .send()
        .await
        .map_err(|e| eyre!("Failed to fetch compiled bundle from {wasm_bucket}/{intermediate_key}: {e}"))?;

    let bytes = object
        .body
        .collect()
        .await
        .map_err(|e| eyre!("Failed to read compiled bundle body: {e}"))?
        .into_bytes();

    let final_key = final_key(fn0_wasmtime_version, subdomain);
    cwasm_s3_client
        .put_object()
        .bucket(cwasm_bucket)
        .key(&final_key)
        .body(aws_sdk_s3::primitives::ByteStream::from(bytes))
        .send()
        .await
        .map_err(|e| eyre!("Failed to upload compiled bundle to {cwasm_bucket}/{final_key}: {e}"))?;

    Ok(())
}

async fn invoke_compile(
    lambda_client: &LambdaClient,
    fn0_wasmtime_version: &str,
    input_bucket: &str,
    input_key: &str,
    output_bucket: &str,
    output_key: &str,
) -> Result<InvokeResult> {
    let function = function_name(fn0_wasmtime_version);
    let payload = InvokePayload {
        input_bucket,
        input_key,
        output_bucket,
        output_key,
    };
    let payload_json = serde_json::to_vec(&payload)?;

    let resp = lambda_client
        .invoke()
        .function_name(&function)
        .payload(Blob::new(payload_json))
        .send()
        .await
        .map_err(|e| eyre!("Lambda invoke {function} failed: {e}"))?;

    if let Some(err) = resp.function_error() {
        let body = resp
            .payload()
            .map(|b| String::from_utf8_lossy(b.as_ref()).to_string())
            .unwrap_or_default();
        return Err(eyre!("Lambda {function} returned error '{err}': {body}"));
    }

    let body = resp
        .payload()
        .ok_or_else(|| eyre!("Lambda {function} returned empty payload"))?;
    let result: InvokeResult = serde_json::from_slice(body.as_ref())
        .map_err(|e| eyre!("Lambda {function} payload parse failed: {e}; body={}", String::from_utf8_lossy(body.as_ref())))?;
    Ok(result)
}
