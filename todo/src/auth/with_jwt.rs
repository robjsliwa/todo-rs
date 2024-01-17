use super::Claims;
use crate::auth::{token_from_header, UserCache};
use crate::error::Error;
use crate::storage::{TodoStore, UserContext};
use jwtverifier::JwtVerifier;
use log::error;
use std::sync::{Arc, Mutex};
use warp::{http::HeaderMap, reject, Filter, Rejection};

pub fn with_jwt(
    jwt_verifier: JwtVerifier,
    store: Arc<dyn TodoStore>,
    cache: Arc<Mutex<UserCache>>,
) -> impl Filter<Extract = (UserContext,), Error = Rejection> + Clone {
    warp::header::headers_cloned()
        .map(move |headers: HeaderMap| {
            (
                headers.clone(),
                jwt_verifier.clone(),
                store.clone(),
                cache.clone(),
            )
        })
        .and_then(
            |(headers, jwt_verifier, store, cache): (
                HeaderMap,
                JwtVerifier,
                Arc<dyn TodoStore>,
                Arc<Mutex<UserCache>>,
            )| async move {
                match token_from_header(&headers) {
                    Ok(jwt) => {
                        let decoded = jwt_verifier.verify::<Claims>(&jwt).await.map_err(|_| {
                            error!("Invalid token");
                            reject::custom(Error::InvalidToken)
                        })?;

                        let external_user_id = decoded.claims.sub;
                        // try to get user from cache first
                        if let Some(user) = cache.lock().unwrap().cache.get(&external_user_id) {
                            return Ok(UserContext {
                                user_id: user.id.clone(),
                                tenant_id: user.tenant_id.clone(),
                            });
                        }

                        // otherwise, try to get user from database
                        match store.get_user(external_user_id.clone()).await {
                            Ok(Some(user)) => {
                                // cahce the user
                                cache
                                    .lock()
                                    .unwrap()
                                    .cache
                                    .put(external_user_id.clone(), user.clone());
                                Ok(UserContext {
                                    user_id: user.id,
                                    tenant_id: user.tenant_id,
                                })
                            }
                            Ok(None) => Err(reject::custom(Error::InvalidToken)),
                            Err(_) => Err(reject::custom(Error::InvalidToken)),
                        }
                    }
                    Err(_) => Err(reject::custom(Error::InvalidToken)),
                }
            },
        )
}
