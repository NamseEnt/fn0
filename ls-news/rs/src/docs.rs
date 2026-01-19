use forte_macros::forte_doc;
use forte_sdk::{anyhow::Result, *};

#[forte_doc]
pub struct UserDoc {
    #[sk]
    pub id: String,
    pub username: String,
    pub avatar_url: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[forte_doc]
pub struct PostDoc {
    #[sk]
    pub id: String,
    pub title: String,
    pub url: String,
    pub content: String,
    pub author_id: String,
    pub likes: usize,
    pub dislikes: usize,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[forte_doc]
pub struct CommentDoc {
    #[pk]
    pub post_id: String,
    #[sk]
    pub id: String,
    pub content: String,
    pub author_id: String,
    pub parent_comment_id: Option<String>,
    pub likes: usize,
    pub dislikes: usize,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[forte_doc]
pub struct DeletedPostDoc {
    #[sk]
    pub id: String,
    pub title: String,
    pub url: String,
    pub content: String,
    pub author_id: String,
    pub likes: usize,
    pub dislikes: usize,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub deleted_at: DateTime,
}

#[forte_doc]
pub struct DeletedCommentDoc {
    #[pk]
    pub post_id: String,
    #[sk]
    pub id: String,
    pub content: String,
    pub author_id: String,
    pub parent_comment_id: Option<String>,
    pub likes: usize,
    pub dislikes: usize,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub deleted_at: DateTime,
}
