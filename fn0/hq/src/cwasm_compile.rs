use color_eyre::eyre::{Result, eyre};
use opendal::Operator;
use serde::{Deserialize, Serialize};

use crate::lambda::LambdaClient;

pub struct BucketRef<'a> {
    pub op: &'a Operator,
    pub name: &'a str,
}

pub struct CompileContext<'a> {
    pub lambda_client: &'a LambdaClient,
    pub wasm_bucket: BucketRef<'a>,
    pub cwasm_bucket: BucketRef<'a>,
    pub fn0_wasmtime_version: &'a str,
}

#[derive(Clone, Copy)]
struct S3Ref<'a> {
    bucket: &'a str,
    key: &'a str,
}

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
    ctx: &CompileContext<'_>,
    subdomain: &str,
    env_present: bool,
) -> Result<()> {
    let input_key = raw_key(subdomain);
    let intermediate_key = intermediate_key(ctx.fn0_wasmtime_version, subdomain);
    let env_key_str = env_key(subdomain);
    let env = env_present.then_some(S3Ref {
        bucket: ctx.wasm_bucket.name,
        key: env_key_str.as_str(),
    });

    invoke_compile(
        ctx.lambda_client,
        ctx.fn0_wasmtime_version,
        S3Ref {
            bucket: ctx.wasm_bucket.name,
            key: &input_key,
        },
        S3Ref {
            bucket: ctx.wasm_bucket.name,
            key: &intermediate_key,
        },
        env,
    )
    .await?;

    let buf = ctx
        .wasm_bucket
        .op
        .read(&intermediate_key)
        .await
        .map_err(|e| {
            eyre!(
                "Failed to fetch compiled bundle from {}/{intermediate_key}: {e}",
                ctx.wasm_bucket.name
            )
        })?;

    let final_key = final_key(ctx.fn0_wasmtime_version, subdomain);
    ctx.cwasm_bucket
        .op
        .write(&final_key, buf)
        .await
        .map_err(|e| {
            eyre!(
                "Failed to upload compiled bundle to {}/{final_key}: {e}",
                ctx.cwasm_bucket.name
            )
        })?;

    Ok(())
}

async fn invoke_compile(
    lambda_client: &LambdaClient,
    fn0_wasmtime_version: &str,
    input: S3Ref<'_>,
    output: S3Ref<'_>,
    env: Option<S3Ref<'_>>,
) -> Result<InvokeResult> {
    let function = function_name(fn0_wasmtime_version);
    let payload = InvokePayload {
        input_bucket: input.bucket,
        input_key: input.key,
        output_bucket: output.bucket,
        output_key: output.key,
        env_bucket: env.map(|e| e.bucket),
        env_key: env.map(|e| e.key),
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

    let result: InvokeResult = serde_json::from_slice(&resp.payload).map_err(|e| {
        eyre!(
            "Lambda {function} payload parse failed: {e}; body={}",
            String::from_utf8_lossy(&resp.payload)
        )
    })?;
    Ok(result)
}
