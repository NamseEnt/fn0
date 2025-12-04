use super::*;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{
    operation::put_item::{PutItemError, builders::PutItemFluentBuilder},
    types::AttributeValue,
};
use chrono::{DateTime, Utc};

pub struct DynamoDbLock {
    client: aws_sdk_dynamodb::Client,
    table_name: String,
}

impl DynamoDbLock {
    pub async fn new() -> Self {
        let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        Self {
            client: aws_sdk_dynamodb::Client::new(&sdk_config),
            table_name: std::env::var("LOCK_TABLE_NAME")
                .expect("env var LOCK_TABLE_NAME is not set"),
        }
    }
    async fn on_no_item(&self, start_time: &DateTime<Utc>) -> Result<bool, anyhow::Error> {
        match self
            .put_item_builder(start_time)
            .condition_expression("attribute_not_exists(master_lock)")
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(err) => {
                if let Some(PutItemError::ConditionalCheckFailedException(_)) =
                    err.as_service_error()
                {
                    return Ok(false);
                }
                Err(err.into())
            }
        }
    }
    fn put_item_builder(&self, start_time: &DateTime<Utc>) -> PutItemFluentBuilder {
        self.client
            .put_item()
            .table_name(&self.table_name)
            .item("master_lock", AttributeValue::S("_".to_string()))
            .item("last_start_time", AttributeValue::N(start_time.timestamp().to_string()))
    }
}

impl Lock for DynamoDbLock {
    fn try_lock<'a>(
        &'a self,
        context: &'a crate::Context,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<bool>> + 'a + Send>> {
        Box::pin(async move {
            let response = self
                .client
                .get_item()
                .table_name(&self.table_name)
                .key("master_lock", AttributeValue::S("_".to_string()))
                .send()
                .await?;

            let Some(item) = response.item else {
                return self.on_no_item(&context.start_time).await;
            };

            let last_start_time = item.get("last_start_time").unwrap().as_n().unwrap();
            let last_start_time = last_start_time.parse::<i64>().unwrap();

            let lock_expired = last_start_time + 30 < context.start_time.timestamp();

            if !lock_expired {
                return Ok(false);
            }

            // optimistic locking
            match self
                .put_item_builder(&context.start_time)
                .condition_expression("last_start_time = :last_start_time")
                .expression_attribute_values(
                    ":last_start_time",
                    AttributeValue::N(last_start_time.to_string()),
                )
                .send()
                .await
            {
                Ok(_) => Ok(true),
                Err(err) => {
                    if let Some(PutItemError::ConditionalCheckFailedException(_)) =
                        err.as_service_error()
                    {
                        return Ok(false);
                    }
                    Err(err.into())
                }
            }
        })
    }
}
