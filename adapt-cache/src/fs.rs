use super::*;
use std::path::PathBuf;

/// FsAdaptCache didn't implemented caching converted data.
#[derive(Clone)]
pub struct FsAdaptCache {
    base_path: PathBuf,
}

impl FsAdaptCache {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

impl<T, E> AdaptCache<T, E> for FsAdaptCache {
    async fn get(
        &self,
        id: &str,
        convert: impl FnOnce(Bytes) -> std::result::Result<(T, usize), E> + Send,
    ) -> std::result::Result<T, Error<E>> {
        let path = self.base_path.join(id);
        match tokio::fs::read(path).await {
            Ok(data) => match convert(Bytes::from(data)) {
                Ok((data, _size)) => Ok(data),
                Err(err) => Err(Error::ConvertError(err)),
            },
            Err(error) => {
                if error.kind() == std::io::ErrorKind::NotFound {
                    return Err(Error::NotFound);
                }
                Err(Error::StorageError(anyhow::anyhow!(error)))
            }
        }
    }
}
