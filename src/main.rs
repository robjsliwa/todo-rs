use crate::error::return_error;
use crate::model::todo::Todo;
use crate::storage::store::TodoStore;
use warp::{
    http::{Method, StatusCode},
    reply::json,
    Filter, Rejection, Reply,
};

mod error;
mod model;
mod storage;

pub fn valid_nanoid() -> impl warp::Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::path::param().and_then(|id: String| async move {
        if id.len() == 9 && id.chars().all(|c| c.is_ascii_alphanumeric()) {
            Ok(id)
        } else {
            Err(warp::reject::custom(error::Error::InvalidId))
        }
    })
}

#[tokio::main]
async fn main() {
    let store = Store::new("./data.json".to_string());
    let store_for_routes = store.clone();
    let store_filter = warp::any().map(move || store_for_routes.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["User-Agent", "Content-Type"])
        .allow_methods(&[Method::GET, Method::POST]);

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
        .with(cors)
        .recover(return_error);

    tokio::select! {
        _ = warp::serve(routes).run(([127, 0, 0, 1], 3030)) => {
            println!("Server started at http://localhost:3030");
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Ctrl-C received, shutting down...");
            store.shutdown().await.unwrap();
        }
    }
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

async fn add_object_handler(store: Store, obj: serde_json::Value) -> Result<impl Reply, Rejection> {
    let mut objects = store.objects.write().await;
    let id = nanoid!(9);
    let store_obj = Object {
        id: id.clone(),
        value: obj,
    };
    objects.insert(id, store_obj);
    Ok(StatusCode::CREATED)
}
