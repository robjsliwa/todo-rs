use warp::{body::BodyDeserializeError, hyper::StatusCode, reject::Reject, Rejection, Reply};

#[derive(Debug, Clone)]
pub enum Error {
    InvalidId,
    NotFound,
    Unauthorized,
    InvalidToken,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::InvalidId => write!(f, "Invalid ID"),
            Error::NotFound => write!(f, "Not found"),
            Error::Unauthorized => write!(f, "Unauthorized"),
            Error::InvalidToken => write!(f, "Invalid token"),
        }
    }
}

impl Reject for Error {}

pub async fn return_error(err: Rejection) -> Result<impl Reply, Rejection> {
    println!("err: {:?}", err);
    let (code, message) = if let Some(error) = err.find::<Error>() {
        match error {
            Error::InvalidId => (StatusCode::BAD_REQUEST, error.to_string()),
            Error::NotFound => (StatusCode::NOT_FOUND, error.to_string()),
            Error::Unauthorized => (StatusCode::UNAUTHORIZED, error.to_string()),
            Error::InvalidToken => (StatusCode::UNAUTHORIZED, error.to_string()),
        }
    } else if let Some(error) = err.find::<BodyDeserializeError>() {
        (StatusCode::UNPROCESSABLE_ENTITY, error.to_string())
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (
            StatusCode::METHOD_NOT_ALLOWED,
            "Method not allowed".to_string(),
        )
    } else if err.find::<warp::reject::UnsupportedMediaType>().is_some() {
        (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Unsupported media type".to_string(),
        )
    } else if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not found".to_string())
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
    };

    Ok(warp::reply::with_status(message, code))
}
