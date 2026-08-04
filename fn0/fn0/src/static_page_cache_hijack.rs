//! Static-page cache hijack: turns a guest's `forte_sdk::static_page_cache::purge`
//! call into a queued invalidation on the project's own zone.
//!
//! The guest names route paths, not URLs. Which host those paths are served at
//! is the project's registered domain, which only control knows, so the worker
//! forwards the paths and control resolves them. That also keeps the worker
//! free of Cloudflare credentials, the same reason public object purges take
//! this route.

use crate::purge_gate::PurgeGate;
use std::sync::Arc;

#[derive(Clone)]
pub struct StaticPageCacheHijack {
    pub placeholder_host: String,
    destination: Destination,
    purge_gate: Option<Arc<PurgeGate>>,
}

#[derive(Clone)]
enum Destination {
    ControlQueue {
        control_project_id: String,
    },
    /// `forte dev` serves every page from the local process, so there is no
    /// edge copy for a purge to invalidate. Accepting and dropping it keeps an
    /// app's publish step running unchanged against the dev server.
    NoEdgeCache,
}

impl StaticPageCacheHijack {
    pub fn new(placeholder_host: String, control_project_id: String) -> Self {
        Self {
            placeholder_host,
            destination: Destination::ControlQueue { control_project_id },
            purge_gate: None,
        }
    }

    pub fn new_local(placeholder_host: String) -> Self {
        Self {
            placeholder_host,
            destination: Destination::NoEdgeCache,
            purge_gate: None,
        }
    }

    /// Shares one budget with public object purges: both spend the same
    /// Cloudflare purge allowance, so one app must not be able to drain it
    /// through whichever of the two is cheaper to call.
    pub fn with_purge_gate(mut self, gate: Arc<PurgeGate>) -> Self {
        self.purge_gate = Some(gate);
        self
    }

    pub fn placeholder_url(&self) -> String {
        format!("http://{}", self.placeholder_host)
    }

    pub fn matches(&self, uri: &hyper::Uri) -> bool {
        uri.host()
            .is_some_and(|host| host.eq_ignore_ascii_case(&self.placeholder_host))
    }

    /// Where a purge is queued, or `None` when nothing holds an edge copy.
    pub(crate) fn control_project_id(&self) -> Option<&str> {
        match &self.destination {
            Destination::ControlQueue { control_project_id } => Some(control_project_id),
            Destination::NoEdgeCache => None,
        }
    }

    /// `true` when the project may spend one more invalidation this hour.
    pub(crate) fn allow_purge(&self, project_id: &str) -> bool {
        match &self.purge_gate {
            Some(gate) => gate.try_purge(project_id, chrono::Utc::now().timestamp() / 3600),
            None => true,
        }
    }

    /// The paths a purge request asks for, rejected as a whole if any one of
    /// them is not a path a static page can be served at.
    ///
    /// Rejecting the batch rather than the offending entry keeps the guest from
    /// believing a partial purge was a whole one.
    ///
    /// Each path is checked against [`crate::static_page::normalize_path`] but
    /// passed on **unchanged**: the edge keys its entry on the URL a visitor
    /// requested, so purging a normalized spelling of a path would clear an
    /// entry nobody has. A path whose spelling normalization would alter is a
    /// rejection, not a rewrite.
    pub(crate) fn parse_paths(&self, body: &[u8]) -> Result<Vec<String>, String> {
        #[derive(serde::Deserialize)]
        struct PurgeRequest {
            paths: Vec<String>,
        }

        let request: PurgeRequest = serde_json::from_slice(body)
            .map_err(|error| format!("malformed purge body: {error}"))?;
        if request.paths.len() > MAX_PATHS_PER_CALL {
            return Err(format!(
                "too many paths in one call; the limit is {MAX_PATHS_PER_CALL}"
            ));
        }
        for path in &request.paths {
            match crate::static_page::normalize_path(path) {
                Ok(normalized) if &normalized == path => {}
                Ok(normalized) => {
                    return Err(format!(
                        "{path}: percent-encoding must be upper-case to match the cached entry; use {normalized}"
                    ));
                }
                Err(error) => return Err(format!("{path}: {error}")),
            }
        }
        Ok(request.paths)
    }
}

/// Bounded so one call cannot hand the queue an unbounded message. Control
/// chunks what it receives against Cloudflare's own per-request ceiling.
const MAX_PATHS_PER_CALL: usize = 100;

#[cfg(test)]
mod tests {
    use super::*;

    fn hijack() -> StaticPageCacheHijack {
        StaticPageCacheHijack::new(
            "fn0-static-page-cache.fn0.dev".to_string(),
            "fn0-control".to_string(),
        )
    }

    #[test]
    fn matches_only_its_own_placeholder_host() {
        let hijack = hijack();
        assert!(
            hijack.matches(
                &"http://fn0-static-page-cache.fn0.dev/purge"
                    .parse()
                    .unwrap()
            )
        );
        assert!(
            hijack.matches(
                &"http://FN0-Static-Page-Cache.fn0.dev/purge"
                    .parse()
                    .unwrap()
            )
        );
        assert!(!hijack.matches(&"http://fn0-public-storage.fn0.dev/purge".parse().unwrap()));
    }

    #[test]
    fn passes_accepted_paths_through_unchanged() {
        assert_eq!(
            hijack()
                .parse_paths(br#"{"paths":["/episode/1","/docs/%7Euser"]}"#)
                .unwrap(),
            vec!["/episode/1".to_string(), "/docs/%7Euser".to_string()]
        );
    }

    #[test]
    fn refuses_a_spelling_normalization_would_change() {
        let error = hijack()
            .parse_paths(br#"{"paths":["/docs/%7euser"]}"#)
            .unwrap_err();
        assert!(error.contains("/docs/%7Euser"), "{error}");
    }

    #[test]
    fn rejects_the_whole_batch_when_one_path_is_unusable() {
        let error = hijack()
            .parse_paths(br#"{"paths":["/episode/1","/episode/1?preview=1"]}"#)
            .unwrap_err();
        assert!(error.contains("/episode/1?preview=1"), "{error}");
    }

    #[test]
    fn refuses_a_batch_over_the_ceiling() {
        let paths: Vec<String> = (0..=MAX_PATHS_PER_CALL)
            .map(|index| format!("/page/{index}"))
            .collect();
        let body = serde_json::json!({ "paths": paths }).to_string();
        assert!(hijack().parse_paths(body.as_bytes()).is_err());
    }
}
