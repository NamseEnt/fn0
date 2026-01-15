use forte_sdk::{anyhow::Result, *};
use serde::{Deserialize, Serialize};

pub trait AsOptStr {
    fn as_opt_str(&self) -> Option<&str>;
}

impl<T: AsRef<str>> AsOptStr for Option<T> {
    fn as_opt_str(&self) -> Option<&str> {
        self.as_ref().map(|s| s.as_ref())
    }
}

impl<T: AsRef<str>> AsOptStr for &Option<T> {
    fn as_opt_str(&self) -> Option<&str> {
        self.as_ref().map(|s| s.as_ref())
    }
}

impl AsOptStr for () {
    fn as_opt_str(&self) -> Option<&str> {
        None
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserDoc {
    pub id: String,
    pub username: String,
    pub avatar_url: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
impl UserDoc {
    pub async fn get(sk: impl AsRef<str>) -> Result<Option<Self>> {
        Ok(forte_db::turso()
            .get("users", sk.as_ref())
            .await?
            .map(|data| serde_json::from_slice(&data))
            .transpose()?)
    }
    pub async fn put(sk: impl AsRef<str>, value: &Self) -> Result<()> {
        forte_db::turso()
            .put("users", sk.as_ref(), &serde_json::to_vec(value)?)
            .await
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Post {
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
impl Post {
    pub async fn get(sk: impl AsRef<str>) -> Result<Option<Self>> {
        Ok(forte_db::turso()
            .get("posts", sk.as_ref())
            .await?
            .map(|data| serde_json::from_slice(&data))
            .transpose()?)
    }
    pub async fn query(after_sk: impl AsOptStr, limit: usize) -> Result<Vec<Self>> {
        Ok(forte_db::turso()
            .query("posts", after_sk.as_opt_str(), limit)
            .await?
            .into_iter()
            .map(|(_sk, data)| serde_json::from_slice(&data))
            .collect::<Result<Vec<_>, _>>()?)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeletedPost {
    pub post: Post,
    pub deleted_at: DateTime,
}
impl DeletedPost {
    pub async fn query(after_sk: impl AsOptStr, limit: usize) -> Result<Vec<Self>> {
        Ok(forte_db::turso()
            .query("deleted_posts", after_sk.as_opt_str(), limit)
            .await?
            .into_iter()
            .map(|(_sk, data)| serde_json::from_slice(&data))
            .collect::<Result<Vec<_>, _>>()?)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Comment {
    pub id: String,
    pub content: String,
    pub post_id: String,
    pub author_id: String,
    pub parent_comment_id: Option<String>,
    pub likes: usize,
    pub dislikes: usize,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
impl Comment {
    pub async fn query(
        pk: impl AsRef<str>,
        after_sk: impl AsOptStr,
        limit: usize,
    ) -> Result<Vec<Self>> {
        Ok(forte_db::turso()
            .query(
                format!("comments#post_id={}", pk.as_ref()),
                after_sk.as_opt_str(),
                limit,
            )
            .await?
            .into_iter()
            .map(|(_sk, data)| serde_json::from_slice(&data))
            .collect::<Result<Vec<_>, _>>()?)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeletedComment {
    pub comment: Comment,
    pub deleted_at: DateTime,
}

impl DeletedComment {
    pub async fn query(
        pk: impl AsRef<str>,
        after_sk: impl AsOptStr,
        limit: usize,
    ) -> Result<Vec<Self>> {
        Ok(forte_db::turso()
            .query(
                format!("deleted_comments#post_id={}", pk.as_ref()),
                after_sk.as_opt_str(),
                limit,
            )
            .await?
            .into_iter()
            .map(|(_sk, data)| serde_json::from_slice(&data))
            .collect::<Result<Vec<_>, _>>()?)
    }
}
