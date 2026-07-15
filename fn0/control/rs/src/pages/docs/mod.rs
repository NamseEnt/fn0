use crate::common::docs_site::{self, NavSection};
use forte_sdk::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct Props {
    pub title: String,
    pub html: String,
    pub nav: Vec<NavSection>,
}

pub async fn handler(_req: ForteRequest<'_>) -> anyhow::Result<Props> {
    let doc = docs_site::find("").expect("docs index must exist");
    Ok(Props {
        title: doc.title.to_string(),
        html: docs_site::render_html(doc),
        nav: docs_site::nav(),
    })
}
