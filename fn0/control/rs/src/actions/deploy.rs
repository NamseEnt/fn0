use crate::common::auth;
use crate::common::aws_sign;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
}

#[derive(Serialize)]
pub enum Output {
    Ok {
        presigned_put_url: String,
        object_key: String,
    },
    NotLoggedIn,
    NotFound,
    Forbidden,
    Error {
        message: String,
    },
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    let Some(user) = auth::bearer_user(req.headers).await else {
        return Output::NotLoggedIn;
    };

    let db = doc_db::turso();
    let project = match (ProjectDocGet {
        project_id: &req.body.project_id,
    })
    .send_with(&db)
    .await
    {
        Ok(Some(p)) => p,
        Ok(None) => return Output::NotFound,
        Err(e) => {
            return Output::Error {
                message: e.to_string(),
            };
        }
    };

    if project.owner_github_id != user.github_id {
        return Output::Forbidden;
    }

    let object_key = format!("original/{}.tar", project.project_id);
    let url = match build_presign(&object_key) {
        Ok(u) => u,
        Err(e) => {
            return Output::Error {
                message: e.to_string(),
            };
        }
    };

    Output::Ok {
        presigned_put_url: url,
        object_key,
    }
}

fn build_presign(key: &str) -> anyhow::Result<String> {
    let account_id = std::env::var("FN0_BUNDLE_STORE_ACCOUNT_ID")
        .map_err(|_| anyhow::anyhow!("FN0_BUNDLE_STORE_ACCOUNT_ID not set"))?;
    let bucket = std::env::var("FN0_BUNDLE_STORE_BUCKET")
        .map_err(|_| anyhow::anyhow!("FN0_BUNDLE_STORE_BUCKET not set"))?;
    let access_key_id = std::env::var("FN0_BUNDLE_STORE_ACCESS_KEY_ID")
        .map_err(|_| anyhow::anyhow!("FN0_BUNDLE_STORE_ACCESS_KEY_ID not set"))?;
    let secret_access_key = std::env::var("FN0_BUNDLE_STORE_SECRET_ACCESS_KEY")
        .map_err(|_| anyhow::anyhow!("FN0_BUNDLE_STORE_SECRET_ACCESS_KEY not set"))?;
    Ok(aws_sign::r2_presign_put(aws_sign::R2PresignArgs {
        account_id: &account_id,
        bucket: &bucket,
        region: "auto",
        key,
        access_key_id: &access_key_id,
        secret_access_key: &secret_access_key,
        expires_seconds: 600,
        now: forte_sdk::now(),
    }))
}
