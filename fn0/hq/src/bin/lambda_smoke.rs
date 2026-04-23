#[path = "../lambda.rs"]
mod lambda;

use color_eyre::eyre::{Result, eyre};
use lambda::LambdaClient;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().ok();

    let region = std::env::var("AWS_REGION").map_err(|_| eyre!("AWS_REGION not set"))?;
    let access_key_id =
        std::env::var("AWS_ACCESS_KEY_ID").map_err(|_| eyre!("AWS_ACCESS_KEY_ID not set"))?;
    let secret_access_key = std::env::var("AWS_SECRET_ACCESS_KEY")
        .map_err(|_| eyre!("AWS_SECRET_ACCESS_KEY not set"))?;
    let function_name =
        std::env::var("LAMBDA_FUNCTION").map_err(|_| eyre!("LAMBDA_FUNCTION not set"))?;
    let payload = std::env::var("LAMBDA_PAYLOAD").unwrap_or_else(|_| "{}".to_string());

    println!(
        "Invoking {function_name} in {region} (payload {} bytes)",
        payload.len()
    );

    let client = LambdaClient::new(&region, &access_key_id, &secret_access_key);
    let resp = client
        .invoke(&function_name, payload.into_bytes())
        .await?;

    println!("function_error: {:?}", resp.function_error);
    println!(
        "payload: {}",
        String::from_utf8_lossy(&resp.payload)
    );

    Ok(())
}
