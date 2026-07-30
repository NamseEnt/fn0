use forte_sdk::ForteRequest;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub name: String,
}

#[derive(Serialize)]
pub struct Output {
    pub greeting: String,
    pub random: u64,
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    Output {
        greeting: format!("Hello, {}!", req.body.name),
        random: forte_sdk::rand::get_random_u64(),
    }
}
