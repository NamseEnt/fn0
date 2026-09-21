pub use crate::bindings::fn0::dibi_transport::client::Error as DibiTransportError;

pub async fn request(endpoint: &str, frame: &[u8]) -> Result<Vec<u8>, DibiTransportError> {
    crate::bindings::fn0::dibi_transport::client::request(endpoint.to_owned(), frame.to_vec()).await
}
