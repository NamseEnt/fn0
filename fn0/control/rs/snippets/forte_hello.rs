#[derive(Serialize)]
pub struct Props {
    pub message: String,
}

pub async fn handler(
    _req: ForteRequest<'_>,
) -> Result<Props> {
    Ok(Props {
        message: "hello from Rust".into(),
    })
}
