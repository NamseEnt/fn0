use crate::common::known_values::{
    PRIVATE_OBJECT_BODY, PRIVATE_OBJECT_KEY, ProbeFailure, probe_response,
};
use anyhow::Result;
use forte_sdk::{ForteRequest, ForteResponse};

pub type Props = ForteResponse;

pub async fn handler(_req: ForteRequest<'_>) -> Result<Props> {
    let result = match object_storage::private::bucket()
        .get(PRIVATE_OBJECT_KEY)
        .await
    {
        Ok(Some(object)) => match object.body.bytes().await {
            Ok(body) if body.as_ref() == PRIVATE_OBJECT_BODY => Ok(()),
            Ok(_) => Err(ProbeFailure::Mismatch),
            Err(error) => {
                eprintln!("storage probe body read failed: {error:?}");
                Err(ProbeFailure::Unavailable)
            }
        },
        Ok(None) => Err(ProbeFailure::Missing),
        Err(error) => {
            eprintln!("storage probe read failed: {error:?}");
            Err(ProbeFailure::Unavailable)
        }
    };
    probe_response(result)
}
