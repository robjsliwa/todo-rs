use crate::error::Error;
use warp::http::header::{HeaderMap, HeaderValue, AUTHORIZATION};

pub fn token_from_header(headers: &HeaderMap<HeaderValue>) -> Result<String, Error> {
    const BEARER: &str = "Bearer ";
    let header = match headers.get(AUTHORIZATION) {
        Some(v) => v,
        None => return Err(Error::Unauthorized),
    };
    let auth_header = match std::str::from_utf8(header.as_bytes()) {
        Ok(v) => v,
        Err(_) => return Err(Error::Unauthorized),
    };
    if !auth_header.starts_with(BEARER) {
        return Err(Error::Unauthorized);
    }
    Ok(auth_header.trim_start_matches(BEARER).to_owned())
}