use crate::error::Error;
use crate::model::todo::{NewTodo, Todo};
use crate::storage::store::{TodoStore, UserContext};
use std::sync::Arc;
use warp::http::StatusCode;

pub async fn add_todo(
    store: Arc<dyn TodoStore>,
    new_todo: NewTodo,
) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(StatusCode::CREATED)
}
