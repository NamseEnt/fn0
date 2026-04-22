use color_eyre::eyre::{Result, eyre};
use opendal::Operator;
use serde::{Deserialize, Serialize};

use crate::lambda::LambdaClient;

#[derive(Serialize)]
struct InvokePayload<'a> {
    input_bucket: &'a str,
    input_key: &'a str,
    output_bucket: &'a str,
    output_key: &'a str,
    env_bucket: Option<&'a str>,
    env_key: Option<&'a str>,
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

pub fn env_key(subdomain: &str) -> String {
    format!("bundles/{subdomain}.env.enc")
}

pub fn intermediate_key(fn0_wasmtime_version: &str, subdomain: &str) -> String {
    format!("compiled/{fn0_wasmtime_version}/{subdomain}.tar.zst")
}

pub fn final_key(fn0_wasmtime_version: &str, subdomain: &str) -> String {
    format!("bundles/{fn0_wasmtime_version}/{subdomain}.tar.zst")
}

pub async fn compile_and_publish(
    lambda_client: &LambdaClient,
    aws_s3: &Operator,
    cwasm_s3: &Operator,
    wasm_bucket: &str,
    cwasm_bucket: &str,
    fn0_wasmtime_version: &str,
    subdomain: &str,
    env_present: bool,
) -> Result<()> {
    let input_key = raw_key(subdomain);
    let intermediate_key = intermediate_key(fn0_wasmtime_version, subdomain);
    let env_key_str = env_key(subdomain);
    let env_key_opt = if env_present { Some(env_key_str.as_str()) } else { None };
    let env_bucket_opt = if env_present { Some(wasm_bucket) } else { None };

    invoke_compile(
        lambda_client,
        fn0_wasmtime_version,
        wasm_bucket,
        &input_key,
        wasm_bucket,
        &intermediate_key,
        env_bucket_opt,
        env_key_opt,
    )
    .await?;

    let buf = aws_s3
        .read(&intermediate_key)
        .await
        .map_err(|e| eyre!("Failed to fetch compiled bundle from {wasm_bucket}/{intermediate_key}: {e}"))?;

    let final_key = final_key(fn0_wasmtime_version, subdomain);
    cwasm_s3
        .write(&final_key, buf)
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
    env_bucket: Option<&str>,
    env_key: Option<&str>,
) -> Result<InvokeResult> {
    let function = function_name(fn0_wasmtime_version);
    let payload = InvokePayload {
        input_bucket,
        input_key,
        output_bucket,
        output_key,
        env_bucket,
        env_key,
    };
    let payload_json = serde_json::to_vec(&payload)?;

    let resp = lambda_client
        .invoke(&function, payload_json)
        .await
        .map_err(|e| eyre!("Lambda invoke {function} failed: {e}"))?;

    if let Some(err) = resp.function_error {
        return Err(eyre!(
            "Lambda {function} returned error '{err}': {}",
            String::from_utf8_lossy(&resp.payload)
        ));
    }

    let result: InvokeResult = serde_json::from_slice(&resp.payload)
        .map_err(|e| eyre!("Lambda {function} payload parse failed: {e}; body={}", String::from_utf8_lossy(&resp.payload)))?;
    Ok(result)
}
