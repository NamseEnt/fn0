pub mod fs;
pub mod s3;

use bytes::Bytes;

pub trait AdaptCache<T, E>: Clone + Send + Sync + 'static {
    fn get(
        &self,
        id: &str,
        convert: impl FnOnce(Bytes) -> std::result::Result<(T, usize), E> + Send,
    ) -> impl Future<Output = Result<T, Error<E>>> + Send;
}

#[derive(Debug)]
pub enum Error<ConvertError> {
    NotFound,
    StorageError(anyhow::Error),
    ConvertError(ConvertError),
    SingleflightLeaderFailed,
}
