use crate::common::auth;
use crate::route_generated::Redirect;
use forte_sdk::*;
use serde::Serialize;

pub struct PathParams {
    pub project_id: String,
    pub trace_id: String,
}

#[derive(Serialize)]
pub enum Props {
    Ok {
        project_id: String,
        name: String,
        trace_id: String,
    },
    NotFound,
}

pub async fn handler(req: ForteRequest<'_>, params: PathParams) -> anyhow::Result<Props> {
    let Some(user) = auth::current_user(req.jar).await else {
        return Err(Redirect::Login.into());
    };
    let owned = user
        .projects
        .into_iter()
        .find(|entry| entry.project_id == params.project_id);
    Ok(match owned {
        Some(entry) => Props::Ok {
            project_id: entry.project_id,
            name: entry.name,
            trace_id: params.trace_id,
        },
        None => Props::NotFound,
    })
}
