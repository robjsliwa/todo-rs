use crate::object::Object;
use crate::error::return_error;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use nanoid::nanoid;
use warp::http::StatusCode;
use warp::reply::json;
use warp::Filter;
use warp::{Rejection, Reply};

mod object;
mod error;

#[derive(Clone)]
struct Store {
    objects: Arc<RwLock<HashMap<String, Object>>>,
}

impl Store {
    fn new() -> Self {
        Store {
            objects: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());
    let id_filter = warp::any().map(|| nanoid!(9));

    let get_object_route = warp::path!("objects" / String)
        .and(warp::get())
        .and(warp::path::end())
        .and(store_filter.clone())
        // .and(id_filter)
        .and_then(get_object_handler);

    let get_objects_route = warp::get()
        .and(warp::path("objects"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(get_objects_handler);

    let add_object_route = warp::path("objects")
        .and(warp::post())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(add_object_handler);

    let routes = get_object_route
        .or(add_object_route)
        .or(get_objects_route)
        .recover(return_error);
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

async fn get_object_handler(id: String, store: Store) -> Result<impl Reply, Rejection> {
    let objects = store.objects.read().await;
    let todo = objects.get(&id);
    println!("id: {}", id );
    match todo {
        Some(value) => Ok(json(&value)),
        None => {
            println!("id not found: {}", id );
            // Err(warp::reject::not_found())
            Err(warp::reject::custom(error::Error::NotFound))
            // Ok(warp::reply::with_status(json(&"not found"), StatusCode::NOT_FOUND))
        },
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
