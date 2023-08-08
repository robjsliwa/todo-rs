use crate::models::Object;
use crate::routes::object_service::ObjectService;
use std::sync::Arc;
use warp::filters::header::header;
use warp::http::StatusCode;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

pub fn object_api<S: ObjectService + Send + Sync + 'static>(
    object_service: Arc<S>,
) -> BoxedFilter<(impl Reply,)> {
    let object_service_filter = warp::any().map(move || Arc::clone(&object_service));

    let get_object = warp::path!("object" / String)
        .and(warp::get())
        .and(header::<String>("authorization"))
        .and(object_service_filter.clone())
        .and_then(get_object_handler);

    let insert_object = warp::path!("object")
        .and(warp::post())
        .and(warp::body::json())
        .and(header::<String>("authorization"))
        .and(object_service_filter.clone())
        .and_then(insert_object_handler);

    let update_object = warp::path!("object")
        .and(warp::put())
        .and(warp::body::json())
        .and(header::<String>("authorization"))
        .and(object_service_filter.clone())
        .and_then(update_object_handler);

    let delete_object = warp::path!("object" / String)
        .and(warp::delete())
        .and(header::<String>("authorization"))
        .and(object_service_filter.clone())
        .and_then(delete_object_handler);

    get_object
        .or(insert_object)
        .or(update_object)
        .or(delete_object)
        .boxed()
}

async fn get_object_handler<S: ObjectService + Send + Sync>(
    id: String,
    token: String,
    object_service: Arc<S>,
) -> Result<impl Reply, Rejection> {
    validate_token(token)?;
    match object_service.get_object(&id).await {
        Some(object) => Ok(warp::reply::json(&object)),
        None => Err(warp::reject::not_found()),
    }
}

async fn insert_object_handler<S: ObjectService + Send + Sync>(
    object: Object,
    token: String,
    object_service: Arc<S>,
) -> Result<impl Reply, Rejection> {
    validate_token(token)?;
    match object_service.insert_object(object).await {
        true => Ok(StatusCode::CREATED),
        false => Ok(StatusCode::BAD_REQUEST),
    }
}

async fn update_object_handler<S: ObjectService + Send + Sync>(
    object: Object,
    token: String,
    object_service: Arc<S>,
) -> Result<impl Reply, Rejection> {
    validate_token(token)?;
    match object_service.update_object(object).await {
        true => Ok(StatusCode::OK),
        false => Ok(StatusCode::BAD_REQUEST),
    }
}

async fn delete_object_handler<S: ObjectService + Send + Sync>(
    id: String,
    token: String,
    object_service: Arc<S>,
) -> Result<impl Reply, Rejection> {
    validate_token(token)?;
    match object_service.delete_object(&id).await {
        true => Ok(StatusCode::OK),
        false => Ok(StatusCode::BAD_REQUEST),
    }
}

fn validate_token(token: String) -> Result<(), Rejection> {
    // Here you would implement your logic to validate the token with Auth0.
    // For simplicity, we will assume the token is always valid.
    Ok(())
}
