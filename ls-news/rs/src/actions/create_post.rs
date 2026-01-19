use crate::common::auth::get_me;
use crate::docs::PostDoc;
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
    InternalError,
    NotLoggedIn,
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    let Some(me) = get_me(req.jar) else {
        return Output::NotLoggedIn;
    };

    let id = Uuid::now_v7().to_string();
    let now = now();

    if let Err(e) = (PostDoc {
        id: id.clone(),
        title: req.body.title,
        url: req.body.url,
        content: req.body.content,
        author_id: me.user_id,
        likes: 0,
        dislikes: 0,
        created_at: now,
        updated_at: now,
    })
    .put()
    .await
    {
        eprintln!("Failed to create post: {e:?}");
        return Output::InternalError;
    }

    Output::Ok { id }
}
