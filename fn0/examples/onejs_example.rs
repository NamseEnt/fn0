// Example: Using fn0 onejs to run JavaScript code
use fn0::onejs;
use http_body_util::{Full, BodyExt};
use hyper::Request;
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple JavaScript handler (already bundled format)
    // User would use rolldown to bundle their code
    let js_code = r#"
        async function handle(req) {
            const url = req.url;
            const method = req.method;
            const bodyText = await req.text();

            console.log(`[${method}] ${url} - Body: ${bodyText}`);

            return new Response(`Hello from fn0 onejs!\n\nMethod: ${method}\nURL: ${url}\nBody: ${bodyText}`, {
                status: 201,
                headers: {
                    "x-custom": "test-value",
                    "content-type": "text/plain",
                    "x-request-method": method,
                }
            });
        }
    "#;

    println!("=== fn0 onejs Example ===\n");

    println!("1. Creating fn0 onejs runtime...");
    let config = onejs::Config { js_code };

    println!("2. Initializing runtime (this creates V8 isolate)...");
    let onejs = onejs::Fn0OneJs::new(config).await?;
    println!("   ✅ Runtime created successfully!\n");

    println!("3. Creating test request...");
    println!("   URL: https://example.com/api/test");
    println!("   Method: POST");
    println!("   Body: \"Hello from Rust!\"\n");

    // Create a simple request with Full<Bytes> body
    let request = Request::builder()
        .method("POST")
        .uri("https://example.com/api/test")
        .body(Full::new(Bytes::from("Hello from Rust!")))?;

    println!("4. Running JavaScript handler...");
    let response = onejs.run(request).await?;

    // Check response
    println!("\n5. Response received:");
    let status = response.status();
    println!("   Status: {}", status);

    println!("   Headers:");
    let headers = response.headers().clone();
    for (name, value) in headers.iter() {
        println!("     {}: {}", name, value.to_str().unwrap_or("<invalid>"));
    }

    // Collect body
    let body_bytes = response.into_body();
    let body = body_bytes.collect().await?.to_bytes();
    println!("   Body:");
    for line in String::from_utf8_lossy(&body).lines() {
        println!("     {}", line);
    }

    // Verify results
    println!("\n6. Verification:");
    assert_eq!(status, 201, "Status should be 201");
    assert_eq!(
        headers.get("x-custom").unwrap(),
        "test-value",
        "x-custom header should match"
    );
    assert!(
        String::from_utf8_lossy(&body).contains("Hello from fn0 onejs!"),
        "Body should contain greeting"
    );
    assert!(
        String::from_utf8_lossy(&body).contains("Hello from Rust!"),
        "Body should echo request body"
    );
    assert!(
        String::from_utf8_lossy(&body).contains("URL: https://example.com/api/test"),
        "Body should contain URL"
    );

    println!("   ✅ All assertions passed!");

    println!("\n=== Test Complete ===");
    Ok(())
}
