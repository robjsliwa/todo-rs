use crate::object::{Object, DB};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::http::StatusCode;
use warp::reply::json;
use warp::Filter;
use warp::Rejection;

mod object;

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
    let id_filter = warp::any().map(|| Uuid::new_v4().to_string());

    let object_handler = warp::get()
        .and(warp::path("object"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(id_filter)
        .and(get_object_handler);

    let add_todo_handler = warp::path("object")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |object: Object| db.clone())
        .and_then(add_object_handler);

    let routes = object_handler.or(add_todo_handler);
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

async fn get_object_handler(store: Store, id: String) -> Result<impl warp::Reply, Rejection> {
    let objects = store.objects.read().await;
    let todo = objects.get(&id);
    match todo {
        Some(value) => Ok(json(&value)),
        None => Err(warp::reject::not_found()),
    }
}

async fn add_object_handler(obj: Object, db: Arc<DB>) -> Result<impl warp::Reply, Rejection> {
    let mut objects = db.write().await;
    objects.insert(obj.id.clone(), obj);
    Ok(StatusCode::CREATED)
}
