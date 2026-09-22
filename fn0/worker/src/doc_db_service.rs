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
        let database = match doc_db::dodb_with_connection(&self.connection, project_id) {
            Ok(database) => database,
            Err(error) => return Box::pin(async move { Err(error.to_string()) }),
        };
        Box::pin(async move {
            database
                .execute_semantic(request)
                .await
                .map_err(|error| error.to_string())
        })
    }
}
