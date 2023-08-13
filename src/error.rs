use warp::{Rejection, Reply, reject::Reject, hyper::StatusCode, body::BodyDeserializeError};

#[derive(Debug)]
pub enum Error {
    InvalidId,
    NotFound,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::InvalidId => write!(f, "Invalid ID"),
            Error::NotFound => write!(f, "Not found"),
        }
    }
}

impl Reject for Error {}

pub async fn return_error(err: Rejection) -> Result<impl Reply, Rejection> {
    let (code, message) = if let Some(error) = err.find::<Error>() {
        match error {
            Error::InvalidId => (StatusCode::BAD_REQUEST, error.to_string()),
            Error::NotFound => (StatusCode::NOT_FOUND, error.to_string()),
        }
    } else if let Some(error) = err.find::<BodyDeserializeError>() {
        (StatusCode::UNPROCESSABLE_ENTITY, error.to_string())
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        (StatusCode::METHOD_NOT_ALLOWED, "Method not allowed".to_string())
    } else if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not found".to_string())
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
    };

    Ok(warp::reply::with_status(message, code))
}
