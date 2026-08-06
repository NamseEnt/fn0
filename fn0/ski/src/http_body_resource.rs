use crate::Body;
use bytes::Bytes;
use deno_core::*;
use deno_error::JsErrorBox;
use http_body_util::BodyExt;
use std::borrow::Cow;
use std::rc::Rc;

pub struct HttpBodyResource {
    state: Rc<AsyncRefCell<ReadState>>,
    cancel: Rc<CancelHandle>,
}

struct ReadState {
    body: Body,
    /// What the current frame still owes the reader. deno_core's default
    /// `read_byob` copies whatever `read` hands back into a fixed-size buffer
    /// without checking that it fits, and it runs inside an op that cannot
    /// unwind — so returning a frame larger than `limit` aborts the worker
    /// process instead of raising a JS error. A frame is therefore handed out
    /// `limit` bytes at a time and the remainder waits here.
    unread: Bytes,
}

impl HttpBodyResource {
    pub fn new(body: Body) -> Self {
        Self {
            state: Rc::new(AsyncRefCell::new(ReadState {
                body,
                unread: Bytes::new(),
            })),
            cancel: CancelHandle::new_rc(),
        }
    }
}

impl Resource for HttpBodyResource {
    fn name(&self) -> Cow<'_, str> {
        "httpBody".into()
    }

    fn close(self: Rc<Self>) {
        self.cancel.cancel();
    }

    fn read(self: Rc<Self>, limit: usize) -> AsyncResult<BufView> {
        let cancel = self.cancel.clone();
        Box::pin(
            async move {
                if limit == 0 {
                    return Ok(BufView::empty());
                }
                let mut state = self.state.borrow_mut().await;
                // An empty BufView is how end of stream is reported, and an
                // empty data frame is not end of stream, so keep pulling.
                while state.unread.is_empty() {
                    match state.body.frame().await {
                        Some(Ok(frame)) => {
                            state.unread = frame.into_data().map_err(|_| {
                                JsErrorBox::generic("Failed to get bytes from frame")
                            })?;
                        }
                        Some(Err(e)) => {
                            return Err(JsErrorBox::generic(format!("Stream error: {}", e)));
                        }
                        None => return Ok(BufView::empty()),
                    }
                }
                let take = limit.min(state.unread.len());
                Ok(BufView::from(state.unread.split_to(take)))
            }
            .try_or_cancel(cancel),
        )
    }
}
