use crate::common::docs_site;
use forte_sdk::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct Props {
    pub rust_snippet_html: String,
    pub props_json_html: String,
    pub tsx_snippet_html: String,
    pub doc_db_snippet_html: String,
    pub object_storage_snippet_html: String,
}

pub async fn handler(_req: ForteRequest<'_>) -> anyhow::Result<Props> {
    Ok(Props {
        rust_snippet_html: docs_site::SNIPPET_FORTE_HELLO_RS_HTML.to_string(),
        props_json_html: docs_site::SNIPPET_FORTE_PROPS_JSON_HTML.to_string(),
        tsx_snippet_html: docs_site::SNIPPET_FORTE_HELLO_TSX_HTML.to_string(),
        doc_db_snippet_html: docs_site::SNIPPET_FORTE_DOC_DB_RS_HTML.to_string(),
        object_storage_snippet_html: docs_site::SNIPPET_FORTE_OBJECT_STORAGE_RS_HTML.to_string(),
    })
}
