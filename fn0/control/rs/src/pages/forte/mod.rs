use forte_sdk::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct Props {}

pub async fn handler(_req: ForteRequest<'_>) -> anyhow::Result<Props> {
    Ok(Props {})
}
