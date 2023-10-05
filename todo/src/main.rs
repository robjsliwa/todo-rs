use crate::commands::invoke_command;
use crate::config::Config;

mod auth;
mod commands;
mod config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    invoke_command(&config);

    Ok(())
}
