pub type Input = crate::queue_task::deployment_activation::Input;

pub async fn handle(input: Input) -> forte_sdk::anyhow::Result<()> {
    crate::queue_task::deployment_activation::handle(input).await
}
