use base64::Engine;
use color_eyre::config::Theme;
use lambda_runtime::{LambdaEvent, service_fn, tracing};
use oci_lambda_proxy::Request;
use oci_rust_sdk::core::{
    RetryConfig,
    auth::{SimpleAuthProvider, SimpleAuthProviderRequiredFields},
    region::Region,
};
use std::{convert::Infallible, env, str::FromStr};

const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

fn main() {
    color_eyre::config::HookBuilder::new()
        .theme(Theme::new())
        .capture_span_trace_by_default(false)
        .add_default_filters()
        .add_frame_filter(Box::new(|frames| {
            frames.retain(|frame| {
                let Some(path) = &frame.filename else {
                    return false;
                };
                !path.to_string_lossy().contains(".cargo")
                    && !path.to_string_lossy().contains(".rustup")
            });
        }))
        .install()
        .unwrap();
    tracing::init_default_subscriber();

    let private_key_base64 =
        env::var("OCI_PRIVATE_KEY_BASE64").expect("env var OCI_PRIVATE_KEY_BASE64 is not set");
    let user_id = env::var("OCI_USER_ID").expect("env var OCI_USER_ID is not set");
    let fingerprint = env::var("OCI_FINGERPRINT").expect("env var OCI_FINGERPRINT is not set");
    let tenancy_id = env::var("OCI_TENANCY_ID").expect("env var OCI_TENANCY_ID is not set");
    let region = env::var("OCI_REGION").expect("env var OCI_REGION is not set");

    let private_key = String::from_utf8_lossy(
        &base64::engine::general_purpose::STANDARD
            .decode(private_key_base64)
            .unwrap(),
    )
    .to_string();

    let region = Region::from_str(&region).unwrap_or_else(|_| {
        panic!("invalid region {region}");
    });

    let auth_provider = SimpleAuthProvider::builder(SimpleAuthProviderRequiredFields {
        tenancy: tenancy_id,
        user: user_id,
        fingerprint,
        private_key,
    })
    .region(region)
    .build();

    let compute = oci_rust_sdk::compute::client(oci_rust_sdk::core::ClientConfig {
        auth_provider,
        region,
        timeout: DEFAULT_TIMEOUT,
        retry: RetryConfig::no_retry(),
    })
    .unwrap();

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let func = service_fn({
            let compute = compute.clone();
            move |event: LambdaEvent<Request>| {
                let compute = compute.clone();
                async move {
                    tracing::info!(?event);
                    println!("event: {event:?}");

                    let output = match event.payload {
                        Request::LaunchInstance(launch_instance_request) => compute
                            .launch_instance(launch_instance_request)
                            .await
                            .map(|res| serde_json::to_string(&res).unwrap())
                            .map_err(|err| err.to_string()),
                        Request::LaunchInstanceConfiguration(
                            launch_instance_configuration_request,
                        ) => compute
                            .launch_instance_configuration(launch_instance_configuration_request)
                            .await
                            .map(|res| serde_json::to_string(&res).unwrap())
                            .map_err(|err| err.to_string()),
                        Request::TerminateInstance(terminate_instance_request) => compute
                            .terminate_instance(terminate_instance_request)
                            .await
                            .map(|res| serde_json::to_string(&res).unwrap())
                            .map_err(|err| err.to_string()),
                        Request::ListInstances(list_instances_request) => compute
                            .list_instances(list_instances_request)
                            .await
                            .map(|res| serde_json::to_string(&res).unwrap())
                            .map_err(|err| err.to_string()),
                    };

                    println!("output: {output:?}");

                    Ok::<_, Infallible>(output)
                }
            }
        });
        lambda_runtime::run(func).await.unwrap();
    });
}
