use crate::docs::*;
use anyhow::Result;
use forte_sdk::*;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

pub struct PathParams {
    pub id: String,
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize)]
pub enum Props {
    Ok {
        post: Post,
        comments: Vec<Comment>,
        users: HashMap<String, UserDoc>,
    },
    // TODO: Return status 404
    NotFound,
    DbErr {
        message: String,
    },
}

pub async fn handler(req: ForteRequest<'_>, path_params: PathParams) -> Result<Props> {
    let is_admin = crate::common::auth::is_admin(req.jar);

    match get_post_with_comments(&path_params.id, is_admin).await {
        Ok(Some((post, comments, users))) => Ok(Props::Ok {
            post,
            comments,
            users,
        }),
        Ok(None) => Ok(Props::NotFound),
        Err(err) => {
            eprintln!("Error: {}", err);
            Ok(Props::DbErr {
                message: "Failed to get post with comments".to_string(),
            })
        }
    }
}

async fn get_post_with_comments(
    post_id: &str,
    is_admin: bool,
) -> Result<Option<(Post, Vec<Comment>, HashMap<String, UserDoc>)>> {
    let Some(post) = Post::get(post_id).await? else {
        return Ok(None);
    };
    let mut comments = Comment::query(post_id, (), 1000).await?;
    if is_admin {
        let deleted_comments = DeletedComment::query(post_id, (), 1000).await?;
        comments.extend(deleted_comments.into_iter().map(|comment| comment.comment));
        comments.sort_by_key(|comment| comment.created_at);
    }

    let user_ids = comments
        .iter()
        .map(|comment| comment.author_id.clone())
        .chain(std::iter::once(post.author_id.clone()))
        .collect::<HashSet<_>>();
    let users = futures::future::try_join_all(user_ids.iter().map(UserDoc::get))
        .await?
        .into_iter()
        .flatten()
        .map(|user| (user.id.clone(), user))
        .collect::<HashMap<_, _>>();
    Ok(Some((post, comments, users)))
}
