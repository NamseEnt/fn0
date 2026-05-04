use crate::common::admin;
use crate::common::aws_sign;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub last_modified: String,
}

#[derive(Serialize)]
pub enum Output {
    Ok,
    Unauthorized,
    Error { message: String },
}

#[derive(Serialize)]
struct InvokePayload<'a> {
    input_bucket: &'a str,
    input_key: String,
    output_bucket: &'a str,
    output_key: String,
    env_bucket: Option<&'a str>,
    env_key: Option<&'a str>,
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if !admin::verify(req.headers) {
        return Output::Unauthorized;
    }

    let db = doc_db::turso();
    let version_doc = match (Fn0WasmtimeVersionDocGet {}).send_with(&db).await {
        Ok(Some(v)) => v,
        Ok(None) => {
            return Output::Error {
                message: "Fn0WasmtimeVersionDoc not set".to_string(),
            };
        }
        Err(e) => {
            return Output::Error {
                message: e.to_string(),
            };
        }
    };

    let mut versions = vec![version_doc.active];
    if let Some(p) = version_doc.pending {
        versions.push(p);
    }

    let env = match read_env() {
        Ok(v) => v,
        Err(e) => {
            return Output::Error {
                message: e.to_string(),
            };
        }
    };

    let input_key = format!("original/{}.tar", req.body.project_id);
    for ver in &versions {
        let output_key = format!(
            "compiled/{ver}/{}/{}.tar.zst",
            req.body.project_id, req.body.last_modified,
        );
        let payload = serde_json::to_vec(&InvokePayload {
            input_bucket: &env.bucket,
            input_key: input_key.clone(),
            output_bucket: &env.bucket,
            output_key,
            env_bucket: None,
            env_key: None,
        })
        .expect("serialize invoke payload");

        let function_name = lambda_function_name(ver);
        if let Err(e) = aws_sign::lambda_invoke_async(aws_sign::LambdaInvokeArgs {
            region: &env.region,
            access_key_id: &env.access_key_id,
            secret_access_key: &env.secret_access_key,
            function_name: &function_name,
            payload,
            now: forte_sdk::now(),
        })
        .await
        {
            return Output::Error {
                message: format!("invoke {function_name}: {e}"),
            };
        }
    }

    Output::Ok
}

struct Env {
    bucket: String,
    region: String,
    access_key_id: String,
    secret_access_key: String,
}

fn read_env() -> anyhow::Result<Env> {
    Ok(Env {
        bucket: std::env::var("FN0_R2_BUCKET")
            .map_err(|_| anyhow::anyhow!("FN0_R2_BUCKET not set"))?,
        region: std::env::var("FN0_LAMBDA_REGION")
            .map_err(|_| anyhow::anyhow!("FN0_LAMBDA_REGION not set"))?,
        access_key_id: std::env::var("FN0_LAMBDA_ACCESS_KEY_ID")
            .map_err(|_| anyhow::anyhow!("FN0_LAMBDA_ACCESS_KEY_ID not set"))?,
        secret_access_key: std::env::var("FN0_LAMBDA_SECRET_ACCESS_KEY")
            .map_err(|_| anyhow::anyhow!("FN0_LAMBDA_SECRET_ACCESS_KEY not set"))?,
    })
}

fn lambda_function_name(fn0_wasmtime_version: &str) -> String {
    format!(
        "fn0-cwasm-compiler-{}",
        fn0_wasmtime_version.replace('.', "-")
    )
}
