use std::cell::Cell;
use std::rc::Rc;

use doc_db::{
    AdminScanRequest, BatchOp, DocGet, DocKey, Document, TrxResult,
};
use forte_sdk::http::{Body, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

mod proxy {
    forte_sdk::wit_bindgen::generate!({
        inline: "package e2e:dibi; world service-export { import wasi:http/types@0.3.0; export wasi:http/handler@0.3.0; }",
        path: "../../../../forte/sdk/wit",
        world: "service-export",
        default_bindings_module: "$crate::proxy",
        pub_export_macro: true,
        features: ["clocks-timezone"],
        with: {
            "wasi:http/handler@0.3.0": generate,
            "wasi:http/types@0.3.0": forte_sdk::bindings::wasi::http::types,
            "wasi:clocks/types@0.3.0": forte_sdk::bindings::wasi::clocks::types,
        },
        runtime_path: "forte_sdk::wit_bindgen::rt",
    });
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct TestDoc {
    key: String,
    value: i64,
}

impl Document for TestDoc {
    fn key(&self) -> DocKey {
        DocKey::new("e2e-doc", self.key.clone())
    }
}

struct TestDocGet {
    key: String,
}

impl DocGet for TestDocGet {
    type Doc = TestDoc;

    fn key(&self) -> DocKey {
        DocKey::new("e2e-doc", self.key.clone())
    }
}

#[derive(Default, Deserialize)]
struct Input {
    value: Option<String>,
}

struct Server;

impl proxy::exports::wasi::http::handler::Guest for Server {
    async fn handle(
        request: forte_sdk::bindings::wasi::http::types::Request,
    ) -> core::result::Result<
        forte_sdk::bindings::wasi::http::types::Response,
        forte_sdk::bindings::wasi::http::types::ErrorCode,
    > {
        forte_sdk::serve::serve(request, |request| async move {
            match dispatch(request).await {
                Ok(response) => Ok(response),
                Err(error) => error_response(error),
            }
        })
        .await
    }
}

proxy::export!(Server);

async fn dispatch(request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let path = request.uri().path().to_owned();
    let input = if matches!(path.as_str(), "/tenant/write") {
        request
            .into_body()
            .json::<Input>()
            .await
            .unwrap_or_default()
    } else {
        Input::default()
    };
    let result = match path.as_str() {
        "/crud" => crud().await,
        "/keys" => keys().await,
        "/batch" => batch().await,
        "/transaction" => explicit_transaction().await,
        "/seed/conflict" => seed_conflict().await,
        "/seed/dependency" => seed_dependency().await,
        "/seed/missing" => seed_missing().await,
        "/seed/readonly" => seed_readonly().await,
        "/trx/success" => trx_success().await,
        "/trx/conflict" => trx_conflict().await,
        "/trx/dependency" => trx_dependency().await,
        "/trx/missing" => trx_missing().await,
        "/trx/readonly" => trx_readonly().await,
        "/tenant/write" => tenant_write(input).await,
        "/tenant/read" => tenant_read().await,
        "/error" => {
            let db = doc_db::dibi();
            db.get("error", "key").await.map(|value| json!(value.is_some()))
        }
        _ => anyhow::bail!("unknown path"),
    }?;
    json_response(result)
}

async fn crud() -> anyhow::Result<Value> {
    let db = doc_db::dibi();
    let missing_before = db.get("crud", "value").await?.is_none();
    db.put("crud", "value", b"first").await?;
    let first = db.get("crud", "value").await?.unwrap_or_default();
    db.put("crud", "value", b"second").await?;
    let second = db.get("crud", "value").await?.unwrap_or_default();
    let binary_data = vec![0, 1, 2, 255, 0];
    db.put("crud", "binary", &binary_data).await?;
    let binary = db.get("crud", "binary").await?.unwrap_or_default();
    db.delete("crud", "value").await?;
    let missing_after = db.get("crud", "value").await?.is_none();
    Ok(json!({
        "missing_before": missing_before,
        "first": first.to_vec(),
        "second": second.to_vec(),
        "binary": binary.to_vec(),
        "missing_after": missing_after,
    }))
}

async fn keys() -> anyhow::Result<Value> {
    let db = doc_db::dibi();
    let keys = ["", "a", "a\0b", "a/b", "a&b", "한글", "日本語", "😀"];
    for key in keys {
        db.put("key-order", key, key.as_bytes()).await?;
    }
    let query = db.query("key-order", None::<&str>, keys.len()).await?;
    let scan = db.scan(None, 100).await?;
    let scan_keys: Vec<String> = scan
        .into_iter()
        .filter(|(pk, _, _)| pk == "key-order")
        .map(|(_, sk, _)| sk)
        .collect();
    Ok(json!({
        "query": query.into_iter().map(|(key, _)| key).collect::<Vec<_>>(),
        "scan": scan_keys,
    }))
}

async fn batch() -> anyhow::Result<Value> {
    let db = doc_db::dibi();
    db.put("batch", "C", b"old").await?;
    db.batch(&[
        BatchOp::Put {
            pk: "batch",
            sk: "A",
            data: b"a",
        },
        BatchOp::Put {
            pk: "batch",
            sk: "B",
            data: b"b",
        },
        BatchOp::Delete { pk: "batch", sk: "C" },
        BatchOp::Put {
            pk: "batch",
            sk: "X",
            data: b"1",
        },
        BatchOp::Put {
            pk: "batch",
            sk: "X",
            data: b"2",
        },
    ])
    .await?;
    let values = ["A", "B", "C", "X"];
    let result = values
        .into_iter()
        .map(|key| {
            let database = db.clone();
            async move { Ok::<_, anyhow::Error>((key, database.get("batch", key).await?)) }
        })
        .collect::<Vec<_>>();
    let mut output = serde_json::Map::new();
    for entry in result {
        let (key, value) = entry.await?;
        output.insert(key.to_owned(), json!(value.map(|bytes| bytes.to_vec())));
    }
    Ok(Value::Object(output))
}

async fn explicit_transaction() -> anyhow::Result<Value> {
    let db = doc_db::dibi();
    db.put("explicit", "D", b"existing").await?;
    let mut transaction = db.transaction().await?;
    transaction.put("explicit", "C", b"temporary").await?;
    transaction.delete("explicit", "D").await?;
    transaction.rollback().await?;
    let outside_c = db.get("explicit", "C").await?;
    let outside_d = db.get("explicit", "D").await?;
    let mut transaction = db.transaction().await?;
    transaction.put("explicit", "A", b"pending").await?;
    let pending_a = transaction.get("explicit", "A").await?;
    transaction.put("explicit", "B", b"committed").await?;
    transaction.commit().await?;
    let committed_a = db.get("explicit", "A").await?;
    let committed_b = db.get("explicit", "B").await?;
    Ok(json!({
        "rollback_c_missing": outside_c.is_none(),
        "rollback_d": outside_d.map(|bytes| bytes.to_vec()),
        "pending_a": pending_a.map(|bytes| bytes.to_vec()),
        "committed_a": committed_a.map(|bytes| bytes.to_vec()),
        "committed_b": committed_b.map(|bytes| bytes.to_vec()),
    }))
}

async fn seed_conflict() -> anyhow::Result<Value> {
    put_doc("conflict", 1).await
}

async fn seed_dependency() -> anyhow::Result<Value> {
    put_doc("dependency-a", 2).await?;
    put_doc("dependency-b", 10).await
}

async fn seed_missing() -> anyhow::Result<Value> {
    let db = doc_db::dibi();
    db.delete("e2e-doc", "missing-a").await?;
    put_doc("missing-b", 10).await
}

async fn seed_readonly() -> anyhow::Result<Value> {
    put_doc("readonly", 1).await
}

async fn put_doc(key: &str, value: i64) -> anyhow::Result<Value> {
    let db = doc_db::dibi();
    let data = serde_json::to_vec(&TestDoc {
        key: key.to_owned(),
        value,
    })?;
    db.put("e2e-doc", key, &data).await?;
    Ok(json!(true))
}

async fn trx_success() -> anyhow::Result<Value> {
    put_doc("success", 1).await?;
    let result = update_doc("success", |doc| doc.value += 1).await?;
    Ok(json!(result))
}

async fn trx_conflict() -> anyhow::Result<Value> {
    let result = update_doc("conflict", |doc| doc.value += 1).await?;
    Ok(json!(result))
}

async fn trx_dependency() -> anyhow::Result<Value> {
    let db = doc_db::dibi();
    let attempts = Rc::new(Cell::new(0));
    let result = db
        .trx({
            let attempts = attempts.clone();
            move |transaction| {
                attempts.set(attempts.get() + 1);
                let attempt_counter = attempts.clone();
                async move {
                    let (a, b) = transaction
                        .get((
                            TestDocGet {
                                key: "dependency-a".to_owned(),
                            },
                            TestDocGet {
                                key: "dependency-b".to_owned(),
                            },
                        ))
                        .await?;
                    let increment = a.as_ref().map(|doc| doc.value).unwrap_or_default();
                    let mut b = b.ok_or_else(|| anyhow::anyhow!("missing dependency B"))?;
                    b.value += increment;
                    drop(a);
                    drop(b);
                    Ok(transaction.commit::<_, ()>(attempt_counter.get())?)
                }
            }
        })
        .await;
    trx_output(result)
}

async fn trx_missing() -> anyhow::Result<Value> {
    let db = doc_db::dibi();
    let attempts = Rc::new(Cell::new(0));
    let result = db
        .trx({
            let attempts = attempts.clone();
            move |transaction| {
                attempts.set(attempts.get() + 1);
                let attempt_counter = attempts.clone();
                async move {
                    let a = transaction
                        .get(TestDocGet {
                            key: "missing-a".to_owned(),
                        })
                        .await?;
                    let mut b = transaction
                        .get(TestDocGet {
                            key: "missing-b".to_owned(),
                        })
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("missing document B"))?;
                    b.value += a.as_ref().map(|doc| doc.value).unwrap_or_default();
                    drop(a);
                    drop(b);
                    Ok(transaction.commit::<_, ()>(attempt_counter.get())?)
                }
            }
        })
        .await;
    trx_output(result)
}

async fn trx_readonly() -> anyhow::Result<Value> {
    let db = doc_db::dibi();
    let attempts = Rc::new(Cell::new(0));
    let result = db
        .trx({
            let attempts = attempts.clone();
            move |transaction| {
                attempts.set(attempts.get() + 1);
                let attempt_counter = attempts.clone();
                async move {
                    let document = transaction
                        .get(TestDocGet {
                            key: "readonly".to_owned(),
                        })
                        .await?;
                    let value = document.as_ref().map(|doc| doc.value);
                    drop(document);
                    Ok(transaction.commit::<_, ()>((attempt_counter.get(), value))?)
                }
            }
        })
        .await;
    match result {
        TrxResult::Committed((attempts, value)) => Ok(json!({
            "attempts": attempts,
            "value": value,
        })),
        TrxResult::Conflict(details) => anyhow::bail!("unexpected conflict: {details:?}"),
        TrxResult::Cancelled(reason) => anyhow::bail!("unexpected cancellation: {reason:?}"),
        TrxResult::Err(error) => Err(error),
    }
}

async fn update_doc<F>(key: &str, update: F) -> anyhow::Result<Value>
where
    F: Fn(&mut TestDoc) + Clone + 'static,
{
    let db = doc_db::dibi();
    let attempts = Rc::new(Cell::new(0));
    let key = key.to_owned();
    let result = db
        .trx({
            let attempts = attempts.clone();
            let key = key.clone();
            move |transaction| {
                let update = update.clone();
                attempts.set(attempts.get() + 1);
                let attempt_counter = attempts.clone();
                let key = key.clone();
                async move {
                    let mut document = transaction
                        .get(TestDocGet { key })
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("missing update document"))?;
                    update(&mut document);
                    let value = document.value;
                    drop(document);
                    Ok(transaction.commit::<_, ()>((attempt_counter.get(), value))?)
                }
            }
        })
        .await;
    match result {
        TrxResult::Committed((attempts, value)) => Ok(json!({
            "attempts": attempts,
            "value": value,
        })),
        TrxResult::Conflict(details) => anyhow::bail!("unexpected conflict: {details:?}"),
        TrxResult::Cancelled(reason) => anyhow::bail!("unexpected cancellation: {reason:?}"),
        TrxResult::Err(error) => Err(error),
    }
}

fn trx_output<Output>(result: TrxResult<Output, (), anyhow::Error>) -> anyhow::Result<Value>
where
    Output: Serialize,
{
    match result {
        TrxResult::Committed(output) => Ok(serde_json::to_value(output)?),
        TrxResult::Conflict(details) => anyhow::bail!("unexpected conflict: {details:?}"),
        TrxResult::Cancelled(reason) => anyhow::bail!("unexpected cancellation: {reason:?}"),
        TrxResult::Err(error) => Err(error),
    }
}

async fn tenant_write(input: Input) -> anyhow::Result<Value> {
    let value = input.value.unwrap_or_else(|| "value".to_owned());
    let db = doc_db::dibi();
    db.put("shared", "key", value.as_bytes()).await?;
    Ok(json!(value))
}

async fn tenant_read() -> anyhow::Result<Value> {
    let db = doc_db::dibi();
    let get = db.get("shared", "key").await?;
    let query = db.query("shared", None::<&str>, 10).await?;
    let scan = db.scan(None, 100).await?;
    let admin = db
        .admin_scan(AdminScanRequest {
            after: None,
            limit: 100,
            pk_prefix: Some("shared".to_owned()),
        })
        .await?;
    Ok(json!({
        "get": get.map(|bytes| bytes.to_vec()),
        "query": query
            .into_iter()
            .map(|(_, value)| value.to_vec())
            .collect::<Vec<_>>(),
        "scan": scan
            .into_iter()
            .filter(|(pk, _, _)| pk == "shared")
            .map(|(_, _, value)| value.to_vec())
            .collect::<Vec<_>>(),
        "admin": admin
            .documents
            .into_iter()
            .map(|document| document.data.to_vec())
            .collect::<Vec<_>>(),
    }))
}

fn json_response(value: Value) -> anyhow::Result<Response<Body>> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&value)?))?)
}

fn error_response(error: anyhow::Error) -> anyhow::Result<Response<Body>> {
    Ok(Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("content-type", "text/plain")
        .body(Body::from(format!("db error: {error:?}")))?)
}
