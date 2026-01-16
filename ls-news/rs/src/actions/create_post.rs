use crate::common::auth::get_me;
use crate::docs::Post;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub title: String,
    pub url: String,
    pub content: String,
    pub post_type: PostType,
}

#[derive(Deserialize, Serialize, Clone)]
pub enum PostType {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "showls")]
    ShowLs,
}

#[derive(Serialize)]
pub enum Output {
    Ok { id: String },
    RateLimitExceeded,
    NotLoggedIn,
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    let Some(me) = get_me(req.jar) else {
        return Output::NotLoggedIn;
    };

    let id = generate_id();
    let now = now();

    let post = Post {
        id: id.clone(),
        title: req.body.title,
        url: req.body.url,
        content: req.body.content,
        author_id: me.user_id,
        likes: 0,
        dislikes: 0,
        created_at: now,
        updated_at: now,
    };

    let sk = format!("{}#{}", now.timestamp_millis(), id);

    if let Err(e) = forte_db::turso()
        .put("posts", &sk, &serde_json::to_vec(&post).unwrap())
        .await
    {
        eprintln!("Failed to create post: {:?}", e);
        return Output::RateLimitExceeded;
    }

    Output::Ok { id }
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{:x}", timestamp)
}
