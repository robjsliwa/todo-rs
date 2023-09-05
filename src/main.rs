use crate::error::return_error;
use crate::routes::{
    add_todo,
    get_todos,
    get_todo,
    update_todo,
    delete_todo,
    uuidv4_param
};
use crate::storage::{
    memstore::MemStore,
    store::TodoStore
};
use auth::with_jwt::with_jwt;
use std::sync::Arc;
use warp::{
    http::Method,
    Filter,
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
    let with_store = warp::any().map(move || store_for_routes.clone());
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET environment variable not set");


    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["User-Agent", "Content-Type", "Authorization"])
        .allow_methods(&[Method::GET, Method::POST, Method::DELETE, Method::PATCH]);

    let get_todo_route = warp::get()
        .and(warp::path("todos"))
        .and(uuidv4_param())
        .and(warp::path::end())
        .and(with_jwt(jwt_secret.clone()))
        .and(with_store.clone())
        .and_then(get_todo);

    let get_todos_route = warp::get()
        .and(warp::path("todos"))
        .and(warp::path::end())
        .and(with_jwt(jwt_secret.clone()))
        .and(with_store.clone())
        .and_then(get_todos);

    let add_todo_route = warp::post()
        .and(warp::path("todos"))
        .and(warp::path::end())
        .and(with_jwt(jwt_secret.clone()))
        .and(with_store.clone())
        .and(warp::body::json())
        .and_then(add_todo);

    let update_todo_route = warp::patch()
        .and(warp::path("todos"))
        .and(uuidv4_param())
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_jwt(jwt_secret.clone()))
        .and(with_store.clone())
        .and_then(update_todo);

    let delete_todo_route = warp::delete()
        .and(warp::path("todos"))
        .and(uuidv4_param())
        .and(warp::path::end())
        .and(with_jwt(jwt_secret.clone()))
        .and(with_store.clone())
        .and_then(delete_todo);

    let routes = get_todo_route
        .or(get_todos_route)
        .or(add_todo_route)
        .or(update_todo_route)
        .or(delete_todo_route)
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
