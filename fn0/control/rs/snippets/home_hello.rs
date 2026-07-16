#[derive(Serialize)]
pub struct Props {
    pub message: String,
}

pub async fn handler(
    _req: ForteRequest<'_>,
) -> Result<Props> {
    Ok(Props {
        message: "typed, server-rendered".into(),
    })
}
