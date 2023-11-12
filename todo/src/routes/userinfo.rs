use crate::auth::UserInfo;
use crate::error::Error;
use crate::storage::TodoStore;
use std::sync::Arc;
use warp::reject;

pub async fn user_info(
    userinfo: UserInfo,
    store: Arc<dyn TodoStore>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let external_user_id = userinfo.sub;
    // get user from database
    match store.get_user(external_user_id.clone()).await {
        Ok(Some(user)) => Ok(warp::reply::json(&user)),
        Ok(None) => Err(reject::custom(Error::NotFound)),
        Err(Error::NotFound) => {
            // create new user and associate with the external user id, new tenant is also created
            let user = store
                .create_user(external_user_id, userinfo.name, userinfo.email)
                .await?;
            Ok(warp::reply::json(&user))
        }
        Err(e) => Err(reject::custom(e)),
    }
}
