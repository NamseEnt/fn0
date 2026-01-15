use crate::Body;
use deno_core::*;
use deno_error::JsErrorBox;
use http_body_util::BodyExt;
use std::borrow::Cow;
use std::rc::Rc;

pub struct HttpBodyResource {
    pub body: Rc<AsyncRefCell<Body>>,
    cancel: Rc<CancelHandle>,
}

impl HttpBodyResource {
    pub fn new(body: Body) -> Self {
        Self {
            body: Rc::new(AsyncRefCell::new(body)),
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

    fn read(self: Rc<Self>, _limit: usize) -> AsyncResult<BufView> {
        let cancel = self.cancel.clone();
        Box::pin(
            async move {
                let mut body = self.body.borrow_mut().await;
                match body.frame().await {
                    Some(Ok(frame)) => frame
                        .into_data()
                        .map(BufView::from)
                        .map_err(|_| JsErrorBox::generic("Failed to get bytes from frame")),
                    Some(Err(e)) => Err(JsErrorBox::generic(format!("Stream error: {}", e))),
                    None => Ok(BufView::empty()),
                }
            }
            .try_or_cancel(cancel),
        )
    }
}
