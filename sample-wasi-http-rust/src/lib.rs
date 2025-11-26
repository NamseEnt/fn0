use wstd::http::body::{BodyForthcoming, IncomingBody};
use wstd::http::server::{Finished, Responder};
use wstd::http::{IntoBody, Request, Response, StatusCode};
use wstd::io::{copy, empty, AsyncWrite};
use wstd::time::{Duration, Instant};

#[wstd::http_server]
async fn main(req: Request<IncomingBody>, res: Responder) -> Finished {
    let path_and_query = req.uri().path_and_query().unwrap().as_str();

    // Extract path without query params for matching
    let path = path_and_query.split('?').next().unwrap();

    match path {
        "/wait" => wait(req, res).await,
        "/echo" => echo(req, res).await,
        "/echo-headers" => echo_headers(req, res).await,
        "/echo-trailers" => echo_trailers(req, res).await,
        "/trap" => trap(req, res).await,
        "/slow" => slow(req, res).await,
        "/error" => error_response(req, res).await,
        "/" => home(req, res).await,
        _ => not_found(req, res).await,
    }
}

async fn home(_req: Request<IncomingBody>, res: Responder) -> Finished {
    // To send a single string as the response body, use `res::respond`.
    res.respond(Response::new("Hello, wasi:http/proxy world!\n".into_body()))
        .await
}

async fn wait(_req: Request<IncomingBody>, res: Responder) -> Finished {
    // Get the time now
    let now = Instant::now();

    // Sleep for one second.
    wstd::task::sleep(Duration::from_secs(1)).await;

    // Compute how long we slept for.
    let elapsed = Instant::now().duration_since(now).as_millis();

    // To stream data to the response body, use `res::start_response`.
    let mut body = res.start_response(Response::new(BodyForthcoming));
    let result = body
        .write_all(format!("slept for {elapsed} millis\n").as_bytes())
        .await;
    Finished::finish(body, result, None)
}

async fn echo(mut req: Request<IncomingBody>, res: Responder) -> Finished {
    // Stream data from the req body to the response body.
    let mut body = res.start_response(Response::new(BodyForthcoming));
    let result = copy(req.body_mut(), &mut body).await;
    Finished::finish(body, result, None)
}

async fn echo_headers(req: Request<IncomingBody>, responder: Responder) -> Finished {
    let mut res = Response::builder();
    *res.headers_mut().unwrap() = req.into_parts().0.headers;
    let res = res.body(empty()).unwrap();
    responder.respond(res).await
}

async fn echo_trailers(req: Request<IncomingBody>, res: Responder) -> Finished {
    let body = res.start_response(Response::new(BodyForthcoming));
    let (trailers, result) = match req.into_body().finish().await {
        Ok(trailers) => (trailers, Ok(())),
        Err(err) => (Default::default(), Err(std::io::Error::other(err))),
    };
    Finished::finish(body, result, trailers)
}

async fn not_found(_req: Request<IncomingBody>, responder: Responder) -> Finished {
    let res = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(empty())
        .unwrap();
    responder.respond(res).await
}

async fn trap(_req: Request<IncomingBody>, _res: Responder) -> Finished {
    // This will cause a WASM trap
    panic!("Intentional trap for testing");
}

async fn slow(req: Request<IncomingBody>, res: Responder) -> Finished {
    // Parse query parameter for sleep duration
    let path_and_query = req.uri().path_and_query().unwrap().as_str();

    // Default to 100ms if no parameter provided
    let mut sleep_ms: u64 = 100;

    // Parse query string
    if let Some(query) = path_and_query.split('?').nth(1) {
        for param in query.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                if key == "ms" {
                    if let Ok(ms) = value.parse::<u64>() {
                        sleep_ms = ms;
                    }
                }
            }
        }
    }

    let now = Instant::now();
    wstd::task::sleep(Duration::from_millis(sleep_ms)).await;
    let elapsed = Instant::now().duration_since(now).as_millis();

    let mut body = res.start_response(Response::new(BodyForthcoming));
    let result = body
        .write_all(format!("slept for {elapsed} millis (requested {sleep_ms} ms)\n").as_bytes())
        .await;
    Finished::finish(body, result, None)
}

async fn error_response(_req: Request<IncomingBody>, responder: Responder) -> Finished {
    // Return an HTTP error status
    let res = Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body("Error endpoint - returns 500".into_body())
        .unwrap();
    responder.respond(res).await
}
