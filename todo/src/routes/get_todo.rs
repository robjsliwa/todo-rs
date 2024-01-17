use crate::storage::store::{TodoStore, UserContext};
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_todo(
    id: Uuid,
    user: UserContext,
    store: Arc<dyn TodoStore>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let todo = store.get_todo(&user, id.to_string()).await?;
    Ok(warp::reply::json(&todo))
}
