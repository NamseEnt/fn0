use wstd::http::body::{BodyForthcoming, IncomingBody};
use wstd::http::server::{Finished, Responder};
use wstd::http::{Client, IntoBody, Request, Response};
use wstd::io::{copy, AsyncWrite};

#[wstd::http_server]
async fn main(_req: Request<IncomingBody>, res: Responder) -> Finished {
    let target = match std::env::var("TARGET_URL") {
        Ok(u) => u,
        Err(_) => {
            return reply(res, 500, b"TARGET_URL not set").await;
        }
    };

    let outbound_req = match Request::builder()
        .method("POST")
        .uri(target.as_str())
        .header("content-type", "application/x-www-form-urlencoded")
        .body("grant_type=authorization_code&code=FAKE".into_body())
    {
        Ok(r) => r,
        Err(e) => return reply(res, 500, format!("build req: {e}").as_bytes()).await,
    };

    let outbound_resp = match Client::new().send(outbound_req).await {
        Ok(r) => r,
        Err(e) => return reply(res, 502, format!("outbound failed: {e}").as_bytes()).await,
    };

    let upstream_status = outbound_resp.status().as_u16();
    let mut body = res.start_response(
        Response::builder()
            .status(upstream_status)
            .body(BodyForthcoming)
            .unwrap(),
    );

    let (_, mut incoming_body) = outbound_resp.into_parts();
    let copy_result = copy(&mut incoming_body, &mut body).await;
    Finished::finish(body, copy_result, None)
}

async fn reply(res: Responder, status: u16, msg: &[u8]) -> Finished {
    let mut body = res.start_response(
        Response::builder()
            .status(status)
            .body(BodyForthcoming)
            .unwrap(),
    );
    let result = body.write_all(msg).await;
    Finished::finish(body, result, None)
}
