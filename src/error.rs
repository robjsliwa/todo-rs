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
    println!("err: {:?}", err);
    if let Some(error) = err.find::<Error>() {
        println!("CHECK1");
        match error {
            Error::NotFound => Ok(warp::reply::with_status(
                "Not Found".to_string(),
                StatusCode::NOT_FOUND,
            )),
            Error::InvalidId => Ok(warp::reply::with_status(
                "Invalid ID".to_string(),
                StatusCode::BAD_REQUEST,
            )),
        }
    } else if let Some(error) = err.find::<BodyDeserializeError>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        Ok(warp::reply::with_status(
            "Method not allowed".to_string(),
            StatusCode::METHOD_NOT_ALLOWED,
        ))
    } else if err.is_not_found() {
        Ok(warp::reply::with_status(
            "Not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Internal server error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}