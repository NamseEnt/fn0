use crate::common::docs_site;
use forte_sdk::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct Props {
    pub forte_snippet_html: String,
}

pub async fn handler(_req: ForteRequest<'_>) -> anyhow::Result<Props> {
    Ok(Props {
        forte_snippet_html: docs_site::SNIPPET_HOME_HELLO_RS_HTML.to_string(),
    })
}
