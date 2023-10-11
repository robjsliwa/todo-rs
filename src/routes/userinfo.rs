use crate::auth::Claims;
use crate::error::Error;
use crate::storage::TodoStore;
use log::error;
use reqwest;
use std::sync::Arc;
use warp::{reject, Rejection};

async fn fetch_user_info(
    external_user_id: &str,
    domain: &str,
) -> Result<(String, String), Rejection> {
    let client = reqwest::Client::new();
    let url = format!("{}/userinfo", domain);
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", external_user_id))
        .send()
        .await
        .map_err(|e| {
            error!("Failed to fetch user info: {:?}", e);
            reject::custom(Error::InvalidToken)
        })?;

    let resp_json: serde_json::Value = resp.json().await.map_err(|e| {
        error!("Failed to fetch user info: {:?}", e);
        reject::custom(Error::InvalidToken)
    })?;

    let name = resp_json["name"]
        .as_str()
        .ok_or_else(|| reject::custom(Error::InvalidToken))?
        .to_string();
    let email = resp_json["email"]
        .as_str()
        .ok_or_else(|| reject::custom(Error::InvalidToken))?
        .to_string();

    Ok((name, email))
}

pub async fn user_info(
    claims: Claims,
    store: Arc<dyn TodoStore>,
    domain: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let external_user_id = claims.sub;
    // get user from database
    match store.get_user(external_user_id.clone()).await {
        Ok(Some(user)) => Ok(warp::reply::json(&user)),
        Ok(None) => {
            // fetch user info from userinfo endpoint
            let (name, email) = fetch_user_info(&external_user_id, &domain).await?;

            // create new user and associate with the external user id, new tenant is also created
            let user = store.create_user(external_user_id, name, email).await?;
            Ok(warp::reply::json(&user))
        }
        Err(_) => Err(reject::custom(Error::InvalidToken)),
    }
}
