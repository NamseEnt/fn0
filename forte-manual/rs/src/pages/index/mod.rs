use anyhow::Result;
use http::{HeaderMap, HeaderValue};
use serde::Serialize;

#[derive(Serialize)]
pub enum Props {
    Ok { text: String },
}

pub async fn handler(_headers: HeaderMap<HeaderValue>) -> Result<Props> {
    let props = Props::Ok {
        text: "Hello, World!".to_string(),
    };
    Ok(props)
}
