use crate::common::known_values::probe_response;
use anyhow::Result;
use forte_sdk::{ForteRequest, ForteResponse};

pub type Props = ForteResponse;

pub async fn handler(_req: ForteRequest<'_>) -> Result<Props> {
    probe_response(Ok(()))
}
