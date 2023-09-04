use crate::storage::store::{TodoStore, UserContext};
use crate::model::todo::UpdateTodo;
use std::sync::Arc;
use uuid::Uuid;

pub async fn update_todo(
    id: Uuid,
    update_todo: UpdateTodo,
    user: UserContext,
    store: Arc<dyn TodoStore>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let todo = store.update_todo(&user, id.to_string(), update_todo).await?;
    Ok(warp::reply::json(&todo))
}