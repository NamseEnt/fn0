use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::combinators::UnsyncBoxBody;
use std::convert::Infallible;
use std::pin::Pin;
use std::task::{Context, Poll};
use wasmtime_wasi_http::body::HyperIncomingBody;

use crate::turso_stats::{ParseJob, submit};

pub fn tee_response(
    resp: hyper::Response<hyper::body::Incoming>,
    code_id: String,
) -> hyper::Response<HyperIncomingBody> {
    let (wasm_tx, wasm_rx) = tokio::sync::mpsc::channel::<Bytes>(8);
    let (parse_tx, parse_rx) = tokio::sync::mpsc::unbounded_channel::<Bytes>();

    submit(ParseJob {
        code_id: code_id.clone(),
        bytes_rx: parse_rx,
    });

    let (parts, upstream_body) = resp.into_parts();
    tokio::spawn(drive_tee(upstream_body, wasm_tx, parse_tx));

    let boxed: HyperIncomingBody =
        UnsyncBoxBody::new(ReceiverBody { rx: wasm_rx }.map_err(|err: Infallible| match err {}));

    hyper::Response::from_parts(parts, boxed)
}

async fn drive_tee(
    mut upstream: hyper::body::Incoming,
    wasm_tx: tokio::sync::mpsc::Sender<Bytes>,
    parse_tx: tokio::sync::mpsc::UnboundedSender<Bytes>,
) {
    while let Some(frame_res) = upstream.frame().await {
        let frame = match frame_res {
            Ok(f) => f,
            Err(err) => {
                tracing::warn!(%err, "turso upstream body error");
                break;
            }
        };
        if let Ok(data) = frame.into_data() {
            let _ = wasm_tx.try_send(data.clone());
            if parse_tx.send(data).is_err() {
                break;
            }
        }
    }
}

struct ReceiverBody {
    rx: tokio::sync::mpsc::Receiver<Bytes>,
}

impl hyper::body::Body for ReceiverBody {
    type Data = Bytes;
    type Error = Infallible;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<hyper::body::Frame<Bytes>, Infallible>>> {
        match self.rx.poll_recv(cx) {
            Poll::Ready(Some(bytes)) => Poll::Ready(Some(Ok(hyper::body::Frame::data(bytes)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
