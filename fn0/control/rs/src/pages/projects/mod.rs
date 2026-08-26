use crate::common::auth;
use crate::route_generated::Redirect;
use forte_sdk::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct ProjectItem {
    pub project_id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct Props {
    pub github_login: String,
    pub projects: Vec<ProjectItem>,
}

pub async fn handler(req: ForteRequest<'_>) -> anyhow::Result<Props> {
    let Some(user) = auth::current_user(req.jar).await else {
        return Err(Redirect::Login.into());
    };
    let projects = user
        .projects
        .into_iter()
        .map(|entry| ProjectItem {
            project_id: entry.project_id,
            name: entry.name,
        })
        .collect();
    Ok(Props {
        github_login: user.github_login,
        projects,
    })
}
