//! Monthly compute egress accounting seen from the runtime.
//!
//! The worker charges every byte a project sends out of the platform: HTTP response bodies,
//! WebSocket message payloads, and request bodies of the application's own outbound HTTP calls.
//! A byte is charged before it is handed to the transport. When the budget refuses a charge the
//! transfer stops immediately; bytes already written stay written. The budget implementation
//! lives in the worker because it talks to the control plane.

use bytes::Bytes;
use http_body::{Body, Frame, SizeHint};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EgressDenied {
    QuotaExhausted,
    QuotaNotConfigured,
    BudgetUnavailable,
}

impl std::fmt::Display for EgressDenied {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::QuotaExhausted => formatter.write_str("monthly egress quota exhausted"),
            Self::QuotaNotConfigured => formatter.write_str("project has no egress quota"),
            Self::BudgetUnavailable => formatter.write_str("egress budget unavailable"),
        }
    }
}

impl std::error::Error for EgressDenied {}

pub type EgressChargeFuture =
    Pin<Box<dyn Future<Output = Result<(), EgressDenied>> + Send + 'static>>;

pub trait EgressBudget: Send + Sync {
    fn charge(&self, project_id: &str, byte_count: u64) -> EgressChargeFuture;

    /// A cheap local answer used to refuse new work up front. `false` does not promise that a
    /// later [`EgressBudget::charge`] succeeds.
    fn known_exhausted(&self, project_id: &str) -> bool;
}

type PendingCharge = (Bytes, EgressChargeFuture);

pub struct EgressMeteredBody<InnerBody, DeniedError>
where
    InnerBody: Body<Data = Bytes>,
{
    inner: Pin<Box<InnerBody>>,
    budget: Arc<dyn EgressBudget>,
    project_id: String,
    denied_error: DeniedError,
    pending_charge: Option<PendingCharge>,
    denied: bool,
}

impl<InnerBody, DeniedError> EgressMeteredBody<InnerBody, DeniedError>
where
    InnerBody: Body<Data = Bytes>,
{
    pub fn new(
        inner: InnerBody,
        budget: Arc<dyn EgressBudget>,
        project_id: String,
        denied_error: DeniedError,
    ) -> Self {
        Self {
            inner: Box::pin(inner),
            budget,
            project_id,
            denied_error,
            pending_charge: None,
            denied: false,
        }
    }
}

impl<InnerBody, DeniedError> Body for EgressMeteredBody<InnerBody, DeniedError>
where
    InnerBody: Body<Data = Bytes>,
    DeniedError: Fn(EgressDenied) -> InnerBody::Error + Unpin,
{
    type Data = Bytes;
    type Error = InnerBody::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        let this = self.get_mut();
        if this.denied {
            return Poll::Ready(None);
        }
        loop {
            if let Some((_, charge)) = this.pending_charge.as_mut() {
                let charge_result = std::task::ready!(charge.as_mut().poll(context));
                let (data, _) = this.pending_charge.take().expect("pending charge exists");
                return match charge_result {
                    Ok(()) => Poll::Ready(Some(Ok(Frame::data(data)))),
                    Err(denied) => {
                        this.denied = true;
                        Poll::Ready(Some(Err((this.denied_error)(denied))))
                    }
                };
            }
            let frame = match std::task::ready!(this.inner.as_mut().poll_frame(context)) {
                Some(Ok(frame)) => frame,
                other => return Poll::Ready(other),
            };
            let data = match frame.into_data() {
                Ok(data) if data.is_empty() => return Poll::Ready(Some(Ok(Frame::data(data)))),
                Ok(data) => data,
                Err(frame) => return Poll::Ready(Some(Ok(frame))),
            };
            let charge = this.budget.charge(&this.project_id, data.len() as u64);
            this.pending_charge = Some((data, charge));
        }
    }

    fn is_end_stream(&self) -> bool {
        self.denied || (self.pending_charge.is_none() && self.inner.is_end_stream())
    }

    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}

#[cfg(test)]
pub(crate) mod test_budget {
    use super::*;
    use std::sync::Mutex;

    pub(crate) struct CountingBudget {
        pub(crate) remaining_bytes: Mutex<u64>,
        pub(crate) charges: Mutex<Vec<(String, u64)>>,
        pub(crate) exhausted: bool,
    }

    impl CountingBudget {
        pub(crate) fn with_remaining(remaining_bytes: u64) -> Arc<Self> {
            Arc::new(Self {
                remaining_bytes: Mutex::new(remaining_bytes),
                charges: Mutex::new(Vec::new()),
                exhausted: false,
            })
        }
    }

    impl EgressBudget for CountingBudget {
        fn charge(&self, project_id: &str, byte_count: u64) -> EgressChargeFuture {
            self.charges
                .lock()
                .unwrap()
                .push((project_id.to_string(), byte_count));
            let mut remaining_bytes = self.remaining_bytes.lock().unwrap();
            let result = if *remaining_bytes >= byte_count {
                *remaining_bytes -= byte_count;
                Ok(())
            } else {
                Err(EgressDenied::QuotaExhausted)
            };
            Box::pin(async move { result })
        }

        fn known_exhausted(&self, _project_id: &str) -> bool {
            self.exhausted
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_budget::CountingBudget;
    use super::*;
    use http_body_util::{BodyExt, StreamBody};

    fn chunked_body(
        chunks: Vec<&'static [u8]>,
    ) -> impl Body<Data = Bytes, Error = String> + Send + 'static {
        StreamBody::new(futures::stream::iter(
            chunks
                .into_iter()
                .map(|chunk| Ok::<_, String>(Frame::data(Bytes::from_static(chunk)))),
        ))
    }

    #[tokio::test]
    async fn charges_each_chunk_before_releasing_it() {
        let budget = CountingBudget::with_remaining(10);
        let mut body = EgressMeteredBody::new(
            chunked_body(vec![b"hello", b"world"]),
            budget.clone(),
            "project".to_string(),
            |denied: EgressDenied| denied.to_string(),
        );
        let first = body.frame().await.unwrap().unwrap().into_data().unwrap();
        assert_eq!(first, Bytes::from_static(b"hello"));
        assert_eq!(budget.charges.lock().unwrap().len(), 1);
        let second = body.frame().await.unwrap().unwrap().into_data().unwrap();
        assert_eq!(second, Bytes::from_static(b"world"));
        assert!(body.frame().await.is_none());
        assert_eq!(*budget.remaining_bytes.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn exact_limit_is_allowed_and_first_chunk_over_limit_stops_the_body() {
        let budget = CountingBudget::with_remaining(5);
        let mut body = EgressMeteredBody::new(
            chunked_body(vec![b"hello", b"!", b"never"]),
            budget.clone(),
            "project".to_string(),
            |denied: EgressDenied| denied.to_string(),
        );
        assert!(body.frame().await.unwrap().is_ok());
        let error = body.frame().await.unwrap().unwrap_err();
        assert_eq!(error, "monthly egress quota exhausted");
        assert!(body.frame().await.is_none());
        assert_eq!(budget.charges.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn empty_body_is_not_charged() {
        let budget = CountingBudget::with_remaining(0);
        let collected = EgressMeteredBody::new(
            chunked_body(vec![b""]),
            budget.clone(),
            "project".to_string(),
            |denied: EgressDenied| denied.to_string(),
        )
        .collect()
        .await
        .unwrap()
        .to_bytes();
        assert!(collected.is_empty());
        assert!(budget.charges.lock().unwrap().is_empty());
    }
}
