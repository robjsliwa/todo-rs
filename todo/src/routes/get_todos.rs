use crate::storage::store::{TodoStore, UserContext};
use std::sync::Arc;

pub async fn get_todos(
    user: UserContext,
    store: Arc<dyn TodoStore>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let todos = store.get_todos(&user).await?;
    Ok(warp::reply::json(&todos))
}
