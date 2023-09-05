use crate::storage::store::{TodoStore, UserContext};
use std::sync::Arc;
use uuid::Uuid;

pub async fn delete_todo(
    id: Uuid,
    user: UserContext,
    store: Arc<dyn TodoStore>,
) -> Result<impl warp::Reply, warp::Rejection> {
    store.delete_todo(&user, id.to_string()).await?;
    Ok(warp::http::StatusCode::NO_CONTENT)
}
