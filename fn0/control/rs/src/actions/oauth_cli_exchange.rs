use crate::common::auth;
use crate::docs::*;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use forte_sdk::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const EXCHANGE_CONFLICT_RETRIES: usize = 8;

#[derive(Deserialize)]
pub struct Input {
    pub code: String,
    pub code_verifier: String,
    pub redirect_uri: Option<String>,
}

#[derive(Serialize)]
pub enum Output {
    Ok { token: String },
    InvalidGrant { message: String },
    Error { message: String },
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    exchange_code(
        &doc_db::database(),
        req.body.code.trim(),
        req.body.code_verifier.trim(),
        req.body.redirect_uri.clone(),
    )
    .await
}

async fn exchange_code(
    database: &doc_db::Database,
    code: &str,
    code_verifier: &str,
    redirect_uri: Option<String>,
) -> Output {
    if code.is_empty() || code_verifier.is_empty() {
        return Output::InvalidGrant {
            message: "authorization code is invalid or expired".to_string(),
        };
    }

    for attempt_number in 0..EXCHANGE_CONFLICT_RETRIES {
        let code = code.to_string();
        let code_verifier = code_verifier.to_string();
        let redirect_uri = redirect_uri.clone();
        let result = database
            .trx(|transaction| {
                let code = code.clone();
                let code_verifier = code_verifier.clone();
                let redirect_uri = redirect_uri.clone();
                async move {
                    let Some(authorization) = transaction
                        .get(CliAuthorizationCodeDocGet { code: code.clone() })
                        .await?
                    else {
                        return transaction.commit::<_, ()>(Output::InvalidGrant {
                            message: "authorization code is invalid or expired".to_string(),
                        });
                    };

                    authorization.delete();

                    if authorization.expires_at < now()
                        || authorization.redirect_uri != redirect_uri
                    {
                        return transaction.commit::<_, ()>(Output::InvalidGrant {
                            message: "authorization code is invalid or expired".to_string(),
                        });
                    }

                    let mut hasher = Sha256::new();
                    hasher.update(code_verifier.as_bytes());
                    let computed_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());
                    if computed_challenge != authorization.code_challenge {
                        return transaction.commit::<_, ()>(Output::InvalidGrant {
                            message: "authorization code is invalid or expired".to_string(),
                        });
                    }

                    let Some(mut user) = transaction
                        .get(UserDocGet {
                            github_id: authorization.github_id,
                        })
                        .await?
                    else {
                        return transaction.commit::<_, ()>(Output::InvalidGrant {
                            message: "authorization code is invalid or expired".to_string(),
                        });
                    };

                    let bytes = rand::get_random_bytes(16);
                    let Ok(uuid_bytes): Result<[u8; 16], _> = bytes.as_slice().try_into() else {
                        return transaction.commit::<_, ()>(Output::Error {
                            message: "rng returned wrong length".to_string(),
                        });
                    };
                    let token_uuid = Uuid::from_bytes(uuid_bytes);
                    let token = match auth::mint_cli_token(user.github_id, &token_uuid) {
                        Ok(token) => token,
                        Err(error) => {
                            return transaction.commit::<_, ()>(Output::Error {
                                message: error.to_string(),
                            });
                        }
                    };

                    user.cli_tokens.push(CliTokenEntry {
                        id: token_uuid.to_string(),
                        label: authorization.label.clone(),
                        created_at: now(),
                    });

                    transaction.commit::<_, ()>(Output::Ok { token })
                }
            })
            .await;

        match result {
            doc_db::TrxResult::Committed(output) => return output,
            doc_db::TrxResult::Cancelled(()) => unreachable!(),
            doc_db::TrxResult::Conflict(error)
                if attempt_number + 1 < EXCHANGE_CONFLICT_RETRIES =>
            {
                tracing::debug!(?error, "oauth_cli_exchange transaction conflict; retrying");
            }
            doc_db::TrxResult::Conflict(error) => {
                return Output::Error {
                    message: format!("authorization exchange conflict: {error:?}"),
                };
            }
            doc_db::TrxResult::Err(error) => {
                return Output::Error {
                    message: error.to_string(),
                };
            }
        }
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::{Output, exchange_code};
    use crate::docs::{
        CliAuthorizationCodeDoc, CliAuthorizationCodeDocGet, CliAuthorizationCodeDocPut, DbRequest,
        UserDoc, UserDocGet, UserDocPut,
    };
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use forte_sdk::{DateTime, now};
    use sha2::{Digest, Sha256};

    fn challenge(verifier: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        URL_SAFE_NO_PAD.encode(hasher.finalize())
    }

    async fn user(database: &doc_db::Database) {
        UserDocPut(UserDoc {
            github_id: 7,
            github_login: "test-user".to_string(),
            created_at: now(),
            cli_tokens: Vec::new(),
            web_sessions: Vec::new(),
            projects: Vec::new(),
        })
        .send_with(database)
        .await
        .unwrap();
    }

    async fn authorization(
        database: &doc_db::Database,
        code: &str,
        verifier: &str,
        redirect_uri: Option<String>,
        expires_at: DateTime,
    ) {
        CliAuthorizationCodeDocPut(CliAuthorizationCodeDoc {
            code: code.to_string(),
            github_id: 7,
            code_challenge: challenge(verifier),
            redirect_uri,
            label: "test-cli".to_string(),
            expires_at,
        })
        .send_with(database)
        .await
        .unwrap();
    }

    #[test]
    fn successful_exchange_consumes_code_and_adds_one_token() {
        futures::executor::block_on(async {
            unsafe {
                std::env::set_var("FN0_TOKEN_HMAC_KEY", "test-key");
            }
            let database = doc_db::memory();
            user(&database).await;
            authorization(
                &database,
                "authorization-code",
                "code-verifier",
                None,
                now() + forte_sdk::chrono::Duration::minutes(5),
            )
            .await;

            let output =
                exchange_code(&database, "authorization-code", "code-verifier", None).await;
            assert!(matches!(output, Output::Ok { .. }));
            assert!(
                (CliAuthorizationCodeDocGet {
                    code: "authorization-code"
                })
                .send_with(&database)
                .await
                .unwrap()
                .is_none()
            );
            let saved_user = (UserDocGet { github_id: 7 })
                .send_with(&database)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(saved_user.cli_tokens.len(), 1);
            assert_eq!(saved_user.cli_tokens[0].label, "test-cli");

            let replay =
                exchange_code(&database, "authorization-code", "code-verifier", None).await;
            assert!(matches!(replay, Output::InvalidGrant { .. }));
        });
    }

    #[test]
    fn invalid_verifier_consumes_code_without_issuing_token() {
        futures::executor::block_on(async {
            let database = doc_db::memory();
            user(&database).await;
            authorization(
                &database,
                "authorization-code",
                "code-verifier",
                None,
                now() + forte_sdk::chrono::Duration::minutes(5),
            )
            .await;

            let output =
                exchange_code(&database, "authorization-code", "wrong-verifier", None).await;
            assert!(matches!(output, Output::InvalidGrant { .. }));
            assert!(
                (CliAuthorizationCodeDocGet {
                    code: "authorization-code"
                })
                .send_with(&database)
                .await
                .unwrap()
                .is_none()
            );
            let saved_user = (UserDocGet { github_id: 7 })
                .send_with(&database)
                .await
                .unwrap()
                .unwrap();
            assert!(saved_user.cli_tokens.is_empty());
        });
    }

    #[test]
    fn legacy_redirect_uri_remains_bound_to_the_code() {
        futures::executor::block_on(async {
            let database = doc_db::memory();
            user(&database).await;
            authorization(
                &database,
                "authorization-code",
                "code-verifier",
                Some("http://127.0.0.1:1234/callback".to_string()),
                now() + forte_sdk::chrono::Duration::minutes(5),
            )
            .await;

            let output =
                exchange_code(&database, "authorization-code", "code-verifier", None).await;
            assert!(matches!(output, Output::InvalidGrant { .. }));
        });
    }
}
