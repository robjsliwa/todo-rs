use super::{token_from_header, Claims, UserInfo};
use crate::error::Error;
use jwtverifier::JwtVerifier;
use log::error;
use warp::{http::HeaderMap, reject, Filter, Rejection};

async fn fetch_user_info(access_token: &str, domain: &str) -> Result<(String, String), Rejection> {
    let client = reqwest::Client::new();
    let url = format!("{}/userinfo", domain);
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", access_token))
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

pub fn with_decoded(
    jwt_verifier: JwtVerifier,
    domain: String,
) -> impl Filter<Extract = (UserInfo,), Error = Rejection> + Clone {
    warp::header::headers_cloned()
        .map(move |headers: HeaderMap| (headers.clone(), jwt_verifier.clone(), domain.clone()))
        .and_then(
            |(headers, jwt_verifier, domain): (HeaderMap, JwtVerifier, String)| async move {
                match token_from_header(&headers) {
                    Ok(jwt) => {
                        let decoded = jwt_verifier.verify::<Claims>(&jwt).await.map_err(|_| {
                            error!("Invalid token");
                            reject::custom(Error::InvalidToken)
                        })?;

                        let (name, email) = fetch_user_info(&jwt, &domain).await?;
                        Ok(UserInfo {
                            sub: decoded.claims.sub,
                            name,
                            email,
                        })
                    }
                    Err(_) => Err(reject::custom(Error::InvalidToken)),
                }
            },
        )
}
