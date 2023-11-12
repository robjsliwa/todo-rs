use log::info;
use std::env;
use std::net::SocketAddr;
use warp::{reply::json, Filter, Rejection, Reply};

#[derive(Debug, Clone)]
struct Config {
    server_addr: SocketAddr,
}

impl Config {
    fn from_env() -> Result<Self, env::VarError> {
        const DEFAULT_ADDR: &str = "0.0.0.0";
        const DEFAULT_PORT: &str = "3031";
        let ip_address = env::var("IDENTITY_ADDR")
            .map(|s| {
                if s.is_empty() {
                    DEFAULT_ADDR.to_string()
                } else {
                    s
                }
            })
            .unwrap_or(DEFAULT_ADDR.to_string());
        let port = env::var("IDENTITY_PORT")
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

        Ok(Self { server_addr })
    }
}

async fn healthz_handler() -> Result<impl Reply, Rejection> {
    Ok(json(&"OK"))
}

#[tokio::main]
async fn main() {
    // This is just placeholder service for Tilt demonstration.
    env_logger::init();

    let config = Config::from_env().expect("Failed to load configuration");

    let healthz_route = warp::path("healthz")
        .and(warp::path::end())
        .and(warp::get())
        .and_then(healthz_handler);
    let routes = healthz_route.with(warp::cors().allow_any_origin());
    info!("Identity server started at {}", config.server_addr);

    tokio::select! {
        _ = warp::serve(routes).run(config.server_addr) => {
            info!("Identity server shutting down...");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl-C received, identity server shutting down...");
        }
    }
}
