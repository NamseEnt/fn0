pub mod dynamodb;

use crate::Context;
use futures::Future;
use std::pin::Pin;

pub trait Lock {
    fn try_lock<'a>(
        &'a self,
        context: &'a Context,
    ) -> Pin<Box<dyn Future<Output = color_eyre::Result<bool>> + 'a + Send>>;
}
