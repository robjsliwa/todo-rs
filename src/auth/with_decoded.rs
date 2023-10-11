use super::Claims;
use crate::auth::token_from_header;
use crate::error::Error;
use jwtverifier::JwtVerifier;
use log::error;
use warp::{http::HeaderMap, reject, Filter, Rejection};

pub fn with_decoded(
    jwt_verifier: JwtVerifier,
) -> impl Filter<Extract = (Claims,), Error = Rejection> + Clone {
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

                        Ok(decoded.claims)
                    }
                    Err(_) => Err(reject::custom(Error::InvalidToken)),
                }
            },
        )
}
