mod utils;

use anyhow::Result;
use http::{HeaderMap, HeaderValue};
use serde::Serialize;

#[derive(Serialize)]
pub enum Props {
    Ok { id: usize },
}

pub async fn handler(_headers: HeaderMap<HeaderValue>, id: usize) -> Result<Props> {
    let props = Props::Ok { id };
    Ok(props)
}
