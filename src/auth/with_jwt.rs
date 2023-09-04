use warp::{
    http::HeaderMap,
    reject,
    Filter, Rejection,
};
use crate::auth::token_from_header::token_from_header;
use crate::error::Error;
use crate::storage::store::UserContext;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    tenant_id: String,
    user_id: String,
    exp: usize,
}

pub fn with_jwt(jwt_secret: String) -> impl Filter<Extract = (UserContext,), Error = Rejection> + Clone {
    warp::header::headers_cloned()
    .map(move |headers: HeaderMap| (headers.clone(), jwt_secret.clone()))
        .and_then(|(headers, jwt_secret): (HeaderMap, String)| async move {
            match token_from_header(&headers) {
                Ok(jwt) => {
                    let decoded = decode::<Claims>(
                        &jwt,
                        &DecodingKey::from_secret(jwt_secret.as_bytes()),
                        &Validation::new(Algorithm::HS256),
                    );

                    match decoded {
                        Ok(data) => Ok(UserContext {
                            tenant_id: data.claims.tenant_id,
                            user_id: data.claims.user_id,
                        }),
                        Err(_) => Err(reject::custom(Error::InvalidToken)),
                    }
                }
                Err(_) => Err(reject::custom(Error::InvalidToken)),
            }
        })
}




