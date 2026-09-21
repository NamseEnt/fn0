use doc_db_protocol::DocDbRequest;
use fn0::{DocDbService, DocDbServiceFuture, TursoHijack};
use std::sync::Arc;

pub struct TursoDocDbService {
    turso_hijack: Arc<TursoHijack>,
}

impl TursoDocDbService {
    pub fn new(turso_hijack: Arc<TursoHijack>) -> Self {
        Self { turso_hijack }
    }

    fn database_url(&self, project_id: &str) -> String {
        format!("https://{}", self.turso_hijack.target_host(project_id))
    }
}

impl DocDbService for TursoDocDbService {
    fn execute<'a>(&'a self, project_id: &'a str, request: DocDbRequest) -> DocDbServiceFuture<'a> {
        let database = doc_db::turso_with_config(
            self.database_url(project_id),
            self.turso_hijack.group_token.clone(),
        );
        Box::pin(async move {
            database
                .execute_semantic(request)
                .await
                .map_err(|error| error.to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_the_project_database_using_turso_hijack_configuration() {
        let hijack = Arc::new(TursoHijack {
            placeholder_host: "fn0-db.fn0.dev".to_string(),
            target_host_suffix: ".turso.example".to_string(),
            group_token: "secret".to_string(),
        });
        let service = TursoDocDbService::new(hijack);
        assert_eq!(
            service.database_url("project"),
            "https://project.turso.example"
        );
    }
}
