use crate::routes::object_handlers::object_api;
use crate::store::object_repository::ObjectRepository;
use std::sync::Arc;

mod models;
mod routes;
mod store;

#[tokio::main]
async fn main() {
    let repository = Arc::new(ObjectRepository::new());
    let api = object_api(repository);

    warp::serve(api).run(([127, 0, 0, 1], 3030)).await;
}
