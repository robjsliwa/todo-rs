use crate::auth::with_jwt;
use crate::routes::router;
use crate::storage::{memstore::MemStore, store::TodoStore};
use std::env;
use std::sync::Arc;

mod auth;
mod error;
mod model;
mod routes;
mod storage;

#[tokio::main]
async fn main() {
    let mem_store = MemStore::new(
        env::var("MEMSTORE_PATH").expect("MEMSTORE_PATH environment variable not set"),
    );
    let store: Arc<dyn TodoStore> = Arc::new(mem_store.clone());
    let store_for_routes = store.clone();
    // let with_store = warp::any().map(move || store.clone());
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET environment variable not set");
    // let with_jwt = with_jwt(jwt_secret.clone());

    tokio::select! {
        _ = warp::serve(router(store_for_routes, with_jwt(jwt_secret))).run(([127, 0, 0, 1], 3030)) => {
            println!("Server started at http://localhost:3030");
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Ctrl-C received, shutting down...");
            mem_store.shutdown().await.unwrap();
        }
    }
}
