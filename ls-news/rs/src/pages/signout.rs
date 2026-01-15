use crate::route_generated::Redirect;
use anyhow::Result;
use forte_sdk::*;

pub type Props = Redirect;

pub async fn handler(req: ForteRequest<'_>) -> Result<Props> {
    crate::common::auth::clear_auth_cookies(req.jar);
    Ok(Redirect::Index)
}
