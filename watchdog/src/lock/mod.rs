pub mod dynamodb;

use futures::Future;
use std::pin::Pin;

pub trait Lock {
    fn try_lock<'a>(&'a self) -> Pin<Box<dyn Future<Output = anyhow::Result<bool>> + 'a + Send>>;
}
