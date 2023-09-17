use crate::auth::with_jwt;
use crate::routes::router;
use crate::storage::{memstore::MemStore, store::TodoStore};
use log::info;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

mod auth;
mod error;
mod model;
mod routes;
mod storage;

#[derive(Debug)]
struct Config {
    memstore_path: String,
    jwt_secret: String,
    server_addr: SocketAddr,
}

impl Config {
    fn from_env() -> Result<Self, env::VarError> {
        let memstore_path = env::var("MEMSTORE_PATH")?;
        let jwt_secret = env::var("JWT_SECRET")?;
        let ip_address = env::var("TODO_ADDR").unwrap_or("0.0.0.0".to_string());
        let port = env::var("TODO_PORT").unwrap_or("3030".to_string());
        let full_addr = format!("{}:{}", ip_address, port);
        let server_addr = full_addr.parse().map_err(|_| env::VarError::NotPresent)?;

        Ok(Self {
            memstore_path,
            jwt_secret,
            server_addr,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = Config::from_env().expect("Failed to load configuration");

    let mem_store = MemStore::new(config.memstore_path);
    let store: Arc<dyn TodoStore> = Arc::new(mem_store.clone());
    let store_for_routes = store.clone();

    tokio::select! {
        _ = warp::serve(router(store_for_routes, with_jwt(config.jwt_secret))).run(config.server_addr) => {
            info!("Server started at {}", config.server_addr);
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl-C received, shutting down...");
            mem_store.shutdown().await?;
        }
    }

    Ok(())
}
