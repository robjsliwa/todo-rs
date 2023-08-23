use crate::object::Object;
use crate::error::return_error;
use crate::store::Store;
use nanoid::nanoid;
use warp::http::StatusCode;
use warp::reply::json;
use warp::Filter;
use warp::{Rejection, Reply};

mod object;
mod error;
mod store;

pub fn valid_nanoid() -> impl warp::Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::path::param()
        .and_then(|id: String| async move {
            if id.len() == 9 && id.chars().all(|c| c.is_ascii_alphanumeric()) {
                Ok(id)
            } else {
                Err(warp::reject::custom(error::Error::InvalidId))
            }
        })
}

#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let get_object_route = warp::get()
        .and(warp::path("objects"))
        .and(valid_nanoid())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(get_object_handler);

    let get_objects_route = warp::get()
        .and(warp::path("objects"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(get_objects_handler);

    let add_object_route = warp::post()
        .and(warp::path("objects"))
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(add_object_handler);

    let routes = get_object_route
        .or(get_objects_route)
        .or(add_object_route)
        .recover(return_error);
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

async fn get_object_handler(id: String, store: Store) -> Result<impl Reply, Rejection> {
    let objects = store.objects.read().await;
    let object = objects.get(&id);
    match object {
        Some(value) => Ok(json(&value)),
        None => Err(warp::reject::custom(error::Error::NotFound)),
    }
}

async fn get_objects_handler(store: Store) -> Result<impl Reply, Rejection> {
    let objects = store.objects.read().await;
    let values: Vec<Object> = objects.values().cloned().collect();
    Ok(json(&values))
}

async fn add_object_handler(store:Store, obj: serde_json::Value) -> Result<impl Reply, Rejection> {
    let mut objects = store.objects.write().await;
    let id = nanoid!(9);
    let store_obj = Object { id: id.clone(), value: obj };
    objects.insert(id, store_obj);
    Ok(StatusCode::CREATED)
}
