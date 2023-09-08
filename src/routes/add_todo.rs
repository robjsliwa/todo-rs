use crate::model::todo::NewTodo;
use crate::storage::store::{TodoStore, UserContext};
use std::sync::Arc;
use warp::http::StatusCode;

pub async fn add_todo(
    user: UserContext,
    store: Arc<dyn TodoStore>,
    new_todo: NewTodo,
) -> Result<impl warp::Reply, warp::Rejection> {
    store.add_todo(&user, new_todo).await?;
    Ok(StatusCode::CREATED)
}
