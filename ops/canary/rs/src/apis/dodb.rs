use crate::common::known_values::{
    DOC_DB_PARTITION_KEY, DOC_DB_SORT_KEY, DOC_DB_VALUE, ProbeFailure, probe_response,
};
use anyhow::Result;
use forte_sdk::{ForteRequest, ForteResponse};

pub type Props = ForteResponse;

pub async fn handler(_req: ForteRequest<'_>) -> Result<Props> {
    let result = match doc_db::database()
        .get(DOC_DB_PARTITION_KEY, DOC_DB_SORT_KEY)
        .await
    {
        Ok(Some(value)) if value.as_ref() == DOC_DB_VALUE => Ok(()),
        Ok(Some(_)) => Err(ProbeFailure::Mismatch),
        Ok(None) => Err(ProbeFailure::Missing),
        Err(error) => {
            eprintln!("dodb probe read failed: {error:?}");
            Err(ProbeFailure::Unavailable)
        }
    };
    probe_response(result)
}
