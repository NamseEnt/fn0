use super::*;
use crate::*;
use sqlx::{Connection, SqliteConnection};
use tokio::sync::Mutex;

pub struct SqliteDocDbHealthRecorder {
    connection: Mutex<SqliteConnection>,
}

impl SqliteDocDbHealthRecorder {
    pub async fn new() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let connection = SqliteConnection::connect(&database_url).await.unwrap();
        Self {
            connection: Mutex::new(connection),
        }
    }
}

impl HealthRecorder for SqliteDocDbHealthRecorder {
    fn read_all<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = color_eyre::Result<HealthRecords>> + 'a + Send>> {
        Box::pin(async move {
            #[derive(sqlx::FromRow)]
            struct Row {
                value: String,
            }
            let row = {
                let mut conn = self.connection.lock().await;
                sqlx::query_as::<_, Row>("SELECT value FROM docs WHERE key = health-records")
                    .fetch_one(&mut *conn)
                    .await?
            };

            Ok(serde_json::from_str::<HealthRecords>(&row.value)?)
        })
    }

    fn write_all<'a>(
        &'a self,
        records: HealthRecords,
    ) -> Pin<Box<dyn Future<Output = color_eyre::Result<()>> + 'a + Send>> {
        Box::pin(async move {
            let json_data = serde_json::to_string(&records)?;

            let mut conn = self.connection.lock().await;
            sqlx::query("INSERT OR REPLACE INTO docs (key, value) VALUES ('health-records', ?)")
                .bind(json_data)
                .execute(&mut *conn)
                .await?;

            Ok(())
        })
    }
}
