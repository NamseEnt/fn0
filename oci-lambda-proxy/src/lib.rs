use aws_config::BehaviorVersion;
use aws_sdk_lambda::primitives::Blob;
use oci_rust_sdk::compute::*;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    LaunchInstance(LaunchInstanceRequest),
    LaunchInstanceConfiguration(LaunchInstanceConfigurationRequest),
    TerminateInstance(TerminateInstanceRequest),
    ListInstances(ListInstancesRequest),
}

pub struct OciLambdaProxy {
    fn_name: String,
    client: aws_sdk_lambda::Client,
}

impl OciLambdaProxy {
    pub async fn new(fn_name: String) -> Self {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let client = aws_sdk_lambda::Client::new(&config);
        Self { fn_name, client }
    }

    async fn call_api<Res: DeserializeOwned>(
        &self,
        request: Request,
    ) -> oci_rust_sdk::core::Result<Res> {
        let request = serde_json::to_string(&request).unwrap();
        println!("request: {request}");

        let response = self
            .client
            .invoke()
            .function_name(&self.fn_name)
            .payload(Blob::new(request))
            .send()
            .await
            .map_err(|err| oci_rust_sdk::core::OciError::Other(err.to_string()))?;

        println!("response: {response:?}");
        println!(
            "payload: {:?}",
            response
                .payload
                .as_ref()
                .map(|x| String::from_utf8_lossy(x.as_ref()).to_string())
        );

        if let Some(function_error) = response.function_error {
            return Err(oci_rust_sdk::core::OciError::Other(function_error));
        }

        let Some(payload) = response.payload else {
            return Err(oci_rust_sdk::core::OciError::Other(
                "no payload".to_string(),
            ));
        };

        let result: std::result::Result<String, String> = serde_json::from_slice(payload.as_ref())
            .map_err(oci_rust_sdk::core::OciError::SerdeError)?;

        match result {
            Ok(string) => Ok(serde_json::from_slice(string.as_ref())
                .map_err(oci_rust_sdk::core::OciError::SerdeError)?),
            Err(e) => Err(oci_rust_sdk::core::OciError::Other(e)),
        }
    }
}

impl Compute for OciLambdaProxy {
    fn launch_instance(
        &self,
        request: LaunchInstanceRequest,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = oci_rust_sdk::core::Result<LaunchInstanceResponse>> + Send + '_>,
    > {
        Box::pin(self.call_api(Request::LaunchInstance(request)))
    }

    fn launch_instance_configuration(
        &self,
        request: LaunchInstanceConfigurationRequest,
    ) -> std::pin::Pin<
        Box<
            dyn Future<Output = oci_rust_sdk::core::Result<LaunchInstanceConfigurationResponse>>
                + Send
                + '_,
        >,
    > {
        Box::pin(self.call_api(Request::LaunchInstanceConfiguration(request)))
    }

    fn terminate_instance(
        &self,
        request: TerminateInstanceRequest,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = oci_rust_sdk::core::Result<TerminateInstanceResponse>> + Send + '_>,
    > {
        Box::pin(self.call_api(Request::TerminateInstance(request)))
    }

    fn list_instances(
        &self,
        request: ListInstancesRequest,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = oci_rust_sdk::core::Result<ListInstancesResponse>> + Send + '_>,
    > {
        Box::pin(self.call_api(Request::ListInstances(request)))
    }
}
