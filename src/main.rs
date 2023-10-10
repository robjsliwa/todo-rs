use crate::auth::with_jwt;
use crate::routes::router;
use crate::storage::{store::TodoStore, MongoStore};
use jwtverifier::JwtVerifier;
use log::{error, info};
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
    server_addr: SocketAddr,
    mongo_uri: String,
    domain: String,
    audience: String,
}

impl Config {
    fn from_env() -> Result<Self, env::VarError> {
        const DEFAULT_ADDR: &str = "0.0.0.0";
        const DEFAULT_PORT: &str = "3030";
        let mongo_uri = env::var("MONGO_URI")?;
        let domain = env::var("AUTH0_DOMAIN")?;
        let audience = env::var("AUTH0_AUDIENCE")?;
        let ip_address = env::var("TODO_ADDR")
            .map(|s| {
                if s.is_empty() {
                    DEFAULT_ADDR.to_string()
                } else {
                    s
                }
            })
            .unwrap_or(DEFAULT_ADDR.to_string());
        let port = env::var("TODO_PORT")
            .map(|s| {
                if s.is_empty() {
                    DEFAULT_PORT.to_string()
                } else {
                    s
                }
            })
            .unwrap_or(DEFAULT_PORT.to_string());
        let full_addr = format!("{}:{}", ip_address, port);
        let server_addr = full_addr.parse().map_err(|_| env::VarError::NotPresent)?;

        Ok(Self {
            server_addr,
            mongo_uri,
            domain,
            audience,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = Config::from_env().expect("Failed to load configuration");

    let mongo_store = MongoStore::init(config.mongo_uri)
        .await
        .unwrap_or_else(|e| {
            error!("Failed to connect to MongoDB: {:?}", e);
            std::process::exit(1);
        });
    let store: Arc<dyn TodoStore> = Arc::new(mongo_store.clone());
    let store_for_routes = store.clone();
    let jwt_verifier = JwtVerifier::new(&config.domain)
        .use_cache(true)
        .validate_aud(&config.audience)
        .build();
    info!("Server started at {}", config.server_addr);

    tokio::select! {
        _ = warp::serve(router(store_for_routes, with_jwt(jwt_verifier.clone()))).run(config.server_addr) => {
            info!("Server shutting down...");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl-C received, shutting down...");
        }
    }

    Ok(())
}
