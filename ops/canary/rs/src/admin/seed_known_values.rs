use crate::common::known_values::{
    DOC_DB_PARTITION_KEY, DOC_DB_SORT_KEY, DOC_DB_VALUE, PRIVATE_OBJECT_BODY,
    PRIVATE_OBJECT_CONTENT_TYPE, PRIVATE_OBJECT_KEY,
};
use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {}

#[derive(Serialize)]
pub struct Output {
    pub doc_db: &'static str,
    pub private_object: &'static str,
}

pub async fn handle(_input: Input) -> anyhow::Result<Output> {
    let database = doc_db::database();
    database
        .put(DOC_DB_PARTITION_KEY, DOC_DB_SORT_KEY, DOC_DB_VALUE)
        .await
        .context("write the known document")?;
    let stored_value = database
        .get(DOC_DB_PARTITION_KEY, DOC_DB_SORT_KEY)
        .await
        .context("read back the known document")?;
    if stored_value.as_deref() != Some(DOC_DB_VALUE) {
        bail!("the known document did not read back as written");
    }

    let bucket = object_storage::private::bucket();
    bucket
        .put(
            PRIVATE_OBJECT_KEY,
            Some(PRIVATE_OBJECT_CONTENT_TYPE),
            PRIVATE_OBJECT_BODY,
        )
        .await
        .context("write the known object")?;
    let stored_object = bucket
        .get(PRIVATE_OBJECT_KEY)
        .await
        .context("read back the known object")?
        .context("the known object is missing right after it was written")?;
    let stored_body = stored_object
        .body
        .bytes()
        .await
        .context("read the known object body")?;
    if stored_body.as_ref() != PRIVATE_OBJECT_BODY {
        bail!("the known object did not read back as written");
    }

    Ok(Output {
        doc_db: "matches",
        private_object: "matches",
    })
}
