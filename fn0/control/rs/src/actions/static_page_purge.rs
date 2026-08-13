use crate::common::auth;
use crate::docs::*;
use crate::route_generated::enqueue;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

/// Bounded so one call cannot hand the queue an unbounded message. The queue
/// task chunks what it receives against Cloudflare's own per-request ceiling.
const MAX_PATHS_PER_CALL: usize = 100;

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    /// Route paths as a visitor requests them, e.g. `/episode/1`.
    pub paths: Vec<String>,
}

#[derive(Serialize)]
pub enum Output {
    Ok { paths: Vec<String> },
    NotLoggedIn,
    NotFound,
    Forbidden,
    NoDomain,
    TooManyPaths { max: usize },
    UnusablePath { path: String, reason: String },
    InternalError { reason: String },
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    let Some(user) = auth::bearer_user(req.headers).await else {
        return Output::NotLoggedIn;
    };

    if req.body.paths.len() > MAX_PATHS_PER_CALL {
        return Output::TooManyPaths {
            max: MAX_PATHS_PER_CALL,
        };
    }
    if let Err((path, reason)) = check_paths(&req.body.paths) {
        return Output::UnusablePath { path, reason };
    }

    let db = doc_db::turso();
    let project = match (ProjectDocGet {
        project_id: &req.body.project_id,
    })
    .send_with(&db)
    .await
    {
        Ok(Some(project)) => project,
        Ok(None) => return Output::NotFound,
        Err(error) => {
            tracing::error!(?error, "static_page_purge ProjectDocGet");
            return Output::InternalError {
                reason: format!("ProjectDocGet: {error}"),
            };
        }
    };

    if project.owner_github_id != user.github_id {
        return Output::Forbidden;
    }

    // Reported rather than queued: the queue task treats a domainless project
    // as a no-op, which from the CLI would read as a purge that worked.
    match (WorkerManifestDocGet {}).send_with(&db).await {
        Ok(Some(manifest)) => {
            let has_domain = manifest
                .project_manifests
                .get(&req.body.project_id)
                .is_some_and(|entry| !entry.domain.is_empty());
            if !has_domain {
                return Output::NoDomain;
            }
        }
        Ok(None) => return Output::NoDomain,
        Err(error) => {
            tracing::error!(?error, "static_page_purge WorkerManifestDocGet");
            return Output::InternalError {
                reason: format!("WorkerManifestDocGet: {error}"),
            };
        }
    }

    if req.body.paths.is_empty() {
        return Output::Ok { paths: Vec::new() };
    }

    if let Err(error) = enqueue::static_page_purge(crate::queue_task::static_page_purge::Input {
        project_id: req.body.project_id.clone(),
        paths: req.body.paths.clone(),
    })
    .await
    {
        tracing::error!(?error, "static_page_purge enqueue");
        return Output::InternalError {
            reason: format!("enqueue: {error}"),
        };
    }

    Output::Ok {
        paths: req.body.paths.clone(),
    }
}

/// Rejects what would not address a page on the project's own domain.
///
/// Lighter than the guest-facing check in the worker's static page cache
/// hijack, and deliberately so: this path is already behind the owner's own
/// credential, so the risk is a typo purging nothing, not an app spending the
/// account's purge budget on paths it does not own.
fn check_paths(paths: &[String]) -> Result<(), (String, String)> {
    for path in paths {
        let reason = if !path.starts_with('/') {
            Some("path must start with '/'")
        } else if path.contains(['?', '#']) {
            Some("query strings and fragments are not allowed")
        } else if path.contains(char::is_whitespace) {
            Some("whitespace is not allowed")
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err((path.clone(), reason.to_string()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check_paths;

    #[test]
    fn accepts_a_plain_route_path() {
        assert!(check_paths(&["/episode/1".to_string(), "/".to_string()]).is_ok());
    }

    #[test]
    fn names_the_path_it_refused() {
        let (path, _) = check_paths(&["/ok".to_string(), "episode/1".to_string()]).unwrap_err();
        assert_eq!(path, "episode/1");
    }

    #[test]
    fn refuses_query_strings_and_whitespace() {
        assert!(check_paths(&["/episode/1?preview=1".to_string()]).is_err());
        assert!(check_paths(&["/episode/ 1".to_string()]).is_err());
    }
}
