//! Reclaims the R2 prefixes of superseded code versions for one project.
//!
//! Both `{project_id}/{code_version}/` prefixes grow with every deploy and
//! nothing else removes them: the static asset bucket holds the deployed
//! frontend build, the static page bucket holds lazily generated page HTML.
//!
//! Enqueued once a deploy reaches `static_cache_state == active`, and it
//! re-reads the manifest rather than trusting the payload, so a redelivered
//! message never prunes against a stale active version.

use crate::common::aws_sign::R2ListedObject;
use crate::common::r2_store::{StaticAssetStore, StaticPageStore};
use crate::docs::*;
use fn0_shared_schema::STATIC_CACHE_STATE_ACTIVE;
use forte_sdk::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};

#[derive(Serialize, Deserialize)]
pub struct Input {
    pub project_id: String,
}

pub async fn handle(input: Input) -> anyhow::Result<()> {
    let db = doc_db::turso();
    let Some(manifest) = (WorkerManifestDocGet {}).send_with(&db).await? else {
        return Ok(());
    };
    let Some(entry) = manifest.project_manifests.get(&input.project_id) else {
        return Ok(());
    };
    // Mid-activation the fleet is split across two code versions, and which
    // one a given worker serves is not observable from here.
    if entry.static_cache_state != STATIC_CACHE_STATE_ACTIVE {
        return Ok(());
    }
    let active_code_version = entry.code_version;
    let now = forte_sdk::now();
    let prefix = format!("{}/", input.project_id);

    let asset_store = StaticAssetStore::from_env()?;
    let mut deleted = 0usize;
    for key in prunable_keys(
        &asset_store.list_all(&prefix, now).await?,
        &input.project_id,
        active_code_version,
    ) {
        asset_store.delete(&key, now).await?;
        deleted += 1;
    }

    let page_store = StaticPageStore::from_env()?;
    for key in prunable_keys(
        &page_store.list_all(&prefix, now).await?,
        &input.project_id,
        active_code_version,
    ) {
        page_store.delete(&key, now).await?;
        deleted += 1;
    }

    tracing::info!(
        project_id = %input.project_id,
        active_code_version,
        deleted,
        "deploy_artifact_prune complete"
    );
    Ok(())
}

/// Keeps every code version at or above the active one, because a deploy
/// uploads its assets before it activates, plus the newest versions below it,
/// because a page already loaded from an earlier deploy is still fetching that
/// deploy's assets. Keys that do not carry a parseable code version are left
/// alone.
///
/// Two versions below active rather than one: a deploy that fails after
/// uploading assets leaves a prefix that never activates, and with a single
/// retained version that dead prefix would evict the version clients are
/// actually still loading from.
const RETAINED_VERSIONS_BELOW_ACTIVE: usize = 2;

fn prunable_keys(
    objects: &[R2ListedObject],
    project_id: &str,
    active_code_version: u64,
) -> Vec<String> {
    let below_active: BTreeSet<u64> = objects
        .iter()
        .filter_map(|object| code_version_of(&object.key, project_id))
        .filter(|code_version| *code_version < active_code_version)
        .collect();
    let retained: HashSet<u64> = below_active
        .iter()
        .rev()
        .take(RETAINED_VERSIONS_BELOW_ACTIVE)
        .copied()
        .collect();

    objects
        .iter()
        .filter(|object| match code_version_of(&object.key, project_id) {
            Some(code_version) => {
                code_version < active_code_version && !retained.contains(&code_version)
            }
            None => false,
        })
        .map(|object| object.key.clone())
        .collect()
}

fn code_version_of(key: &str, project_id: &str) -> Option<u64> {
    key.strip_prefix(project_id)?
        .strip_prefix('/')?
        .split('/')
        .next()?
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::prunable_keys;
    use crate::common::aws_sign::R2ListedObject;

    fn object(key: &str) -> R2ListedObject {
        R2ListedObject {
            key: key.to_string(),
            size: 0,
            last_modified: forte_sdk::chrono::DateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn keeps_active_and_two_below_and_prunes_older() {
        let objects = vec![
            object("proj/100/client.js"),
            object("proj/200/client.js"),
            object("proj/300/client.js"),
            object("proj/400/client.js"),
            object("proj/400/__forte/pages/AB.html"),
        ];
        assert_eq!(
            prunable_keys(&objects, "proj", 400),
            vec!["proj/100/client.js".to_string()]
        );
    }

    // The failed deploy leaves a prefix that never activates; it must not
    // evict the version clients are still loading from.
    #[test]
    fn a_failed_deploy_below_active_does_not_evict_the_live_previous_version() {
        let objects = vec![
            object("proj/100/client.js"),
            object("proj/200/client.js"),
            object("proj/300/partial-upload.js"),
            object("proj/400/client.js"),
        ];
        assert_eq!(
            prunable_keys(&objects, "proj", 400),
            vec!["proj/100/client.js".to_string()]
        );
    }

    #[test]
    fn keeps_versions_newer_than_active() {
        let objects = vec![
            object("proj/100/client.js"),
            object("proj/200/client.js"),
            object("proj/300/client.js"),
        ];
        assert_eq!(
            prunable_keys(&objects, "proj", 200),
            Vec::<String>::new(),
            "300 is a deploy that uploaded before activating"
        );
    }

    #[test]
    fn leaves_keys_without_a_code_version() {
        let objects = vec![object("proj/not-a-version/client.js"), object("proj/x")];
        assert_eq!(prunable_keys(&objects, "proj", 300), Vec::<String>::new());
    }
}
