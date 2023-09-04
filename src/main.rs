use crate::error::return_error;
use crate::routes::add_todo::add_todo;
use crate::storage::{memstore::MemStore, store::TodoStore};
use auth::with_jwt::with_jwt;
use std::sync::Arc;
use warp::{
    http::{Method, StatusCode},
    reply::json,
    Filter, Rejection, Reply,
};
use std::env;

mod error;
mod model;
mod routes;
mod storage;
mod auth;

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
    let mem_store = MemStore::new("./data.json".to_string());
    let store: Arc<dyn TodoStore> = Arc::new(mem_store.clone());
    let store_for_routes = store.clone();
    let store_filter = warp::any().map(move || store_for_routes.clone());
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET environment variable not set");


    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["User-Agent", "Content-Type", "Authorization"])
        .allow_methods(&[Method::GET, Method::POST]);

    // let get_object_route = warp::get()
    //     .and(warp::path("objects"))
    //     .and(valid_nanoid())
    //     .and(warp::path::end())
    //     .and(store_filter.clone())
    //     .and_then(get_object_handler);

    // let get_objects_route = warp::get()
    //     .and(warp::path("objects"))
    //     .and(warp::path::end())
    //     .and(store_filter.clone())
    //     .and_then(get_objects_handler);

    let add_todo_route = warp::post()
        .and(warp::path("todos"))
        .and(with_jwt(jwt_secret.clone()))
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(add_todo);

    let routes = add_todo_route
        // .or(get_objects_route)
        // .or(add_object_route)
        .with(cors)
        .recover(return_error);

    tokio::select! {
        _ = warp::serve(routes).run(([127, 0, 0, 1], 3030)) => {
            println!("Server started at http://localhost:3030");
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Ctrl-C received, shutting down...");
            mem_store.shutdown().await.unwrap();
        }
    }
}

// async fn get_object_handler(id: String, store: Store) -> Result<impl Reply, Rejection> {
//     let objects = store.objects.read().await;
//     let object = objects.get(&id);
//     match object {
//         Some(value) => Ok(json(&value)),
//         None => Err(warp::reject::custom(error::Error::NotFound)),
//     }
// }

// async fn get_objects_handler(store: Store) -> Result<impl Reply, Rejection> {
//     let objects = store.objects.read().await;
//     let values: Vec<Object> = objects.values().cloned().collect();
//     Ok(json(&values))
// }

// async fn add_object_handler(store: Store, obj: serde_json::Value) -> Result<impl Reply, Rejection> {
//     let mut objects = store.objects.write().await;
//     let id = nanoid!(9);
//     let store_obj = Object {
//         id: id.clone(),
//         value: obj,
//     };
//     objects.insert(id, store_obj);
//     Ok(StatusCode::CREATED)
// }
