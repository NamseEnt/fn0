use crate::common::docs_site::{self, NavSection};
use forte_sdk::*;
use serde::Serialize;

pub struct PathParams {
    pub section: String,
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

pub async fn cache_static_eligible(params: PathParams) -> anyhow::Result<bool> {
    Ok(docs_site::find(&format!("{}/{}", params.section, params.page)).is_some())
}

#[forte_sdk::cache_static]
pub async fn handler(_req: ForteRequest<'_>, params: PathParams) -> anyhow::Result<Props> {
    let route = format!("{}/{}", params.section, params.page);
    Ok(match docs_site::find(&route) {
        Some(doc) => Props::Ok {
            title: doc.title.to_string(),
            html: doc.html.to_string(),
            nav: docs_site::nav(),
        },
        None => Props::NotFound {
            nav: docs_site::nav(),
        },
    })
}
