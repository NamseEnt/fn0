use doc_db_protocol::DocDbRequest;
use fn0::{DocDbService, DocDbServiceFuture};

pub struct DodbDocDbService {
    connection: doc_db::DodbConnection,
}

impl DodbDocDbService {
    pub fn new(connection: doc_db::DodbConnection) -> Self {
        Self { connection }
    }
}

impl DocDbService for DodbDocDbService {
    fn execute<'a>(&'a self, project_id: &'a str, request: DocDbRequest) -> DocDbServiceFuture<'a> {
        let database = doc_db::dodb_with_connection(&self.connection, project_id);
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
    #[test]
    fn project_tenant_mapping_is_stable_and_scoped() {
        assert_eq!(
            doc_db::project_tenant_id("project"),
            doc_db::project_tenant_id("project")
        );
        assert_ne!(
            doc_db::project_tenant_id("project-a"),
            doc_db::project_tenant_id("project-b")
        );
    }
}
