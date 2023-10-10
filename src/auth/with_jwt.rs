use super::Claims;
use crate::auth::token_from_header::token_from_header;
use crate::error::Error;
use crate::storage::store::UserContext;
use jwtverifier::JwtVerifier;
use log::error;
use warp::{http::HeaderMap, reject, Filter, Rejection};

pub fn with_jwt(
    jwt_verifier: JwtVerifier,
) -> impl Filter<Extract = (UserContext,), Error = Rejection> + Clone {
    warp::header::headers_cloned()
        .map(move |headers: HeaderMap| (headers.clone(), jwt_verifier.clone()))
        .and_then(
            |(headers, jwt_verifier): (HeaderMap, JwtVerifier)| async move {
                match token_from_header(&headers) {
                    Ok(jwt) => {
                        let decoded = jwt_verifier.verify::<Claims>(&jwt).await.map_err(|_| {
                            error!("Invalid token");
                            reject::custom(Error::InvalidToken)
                        })?;
                        let user_context = UserContext {
                            user_id: decoded.claims.sub,
                            tenant_id: "1".to_string(),
                        };
                        Ok(user_context)
                    }
                    Err(_) => Err(reject::custom(Error::InvalidToken)),
                }
            },
        )
}
