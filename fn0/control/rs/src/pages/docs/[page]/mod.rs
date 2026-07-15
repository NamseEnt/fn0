use crate::common::docs_site::{self, NavSection};
use forte_sdk::*;
use serde::Serialize;

pub struct PathParams {
    pub page: String,
}

#[derive(Serialize)]
pub enum Props {
    Ok {
        title: String,
        html: String,
        nav: Vec<NavSection>,
    },
    NotFound {
        nav: Vec<NavSection>,
    },
}

pub async fn handler(_req: ForteRequest<'_>, params: PathParams) -> anyhow::Result<Props> {
    Ok(match docs_site::find(&params.page) {
        Some(doc) => Props::Ok {
            title: doc.title.to_string(),
            html: docs_site::render_html(doc),
            nav: docs_site::nav(),
        },
        None => Props::NotFound {
            nav: docs_site::nav(),
        },
    })
}
