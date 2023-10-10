use crate::commands::{invoke_command, CommandContext};
use crate::config::Config;
use cred_store::Credentials;

mod auth;
mod commands;
mod config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let mut credentials = Credentials::new()
        .set_file_name(".credentials".to_string())
        .build()
        .load()?;

    let mut context = CommandContext {
        config: &config,
        cred_store: &mut credentials,
    };

    invoke_command(&mut context);

    Ok(())
}
