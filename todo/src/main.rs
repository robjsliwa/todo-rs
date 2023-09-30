use crate::config::Config;
use clap::{Parser, Subcommand};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

mod config;

#[derive(Parser)]
#[clap(author, version, about = "A command line tool for managing todos")]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Login,
    Logout,
    #[clap(subcommand)]
    Todos(TodosCommand),
}

#[derive(Subcommand)]
enum TodosCommand {
    View(TodoViewCommand),
    List,
    Add(TodoAddCommand),
    Complete(TodoCompleteCommand),
}

#[derive(Parser)]
struct TodoViewCommand {
    #[arg(long = "todo-id")]
    todo_id: u32,
}

#[derive(Parser)]
struct TodoAddCommand {
    #[arg(long = "todo-name")]
    todo_name: String,
}

#[derive(Parser)]
struct TodoCompleteCommand {
    #[arg(long = "todo-id", exclusive = true)]
    todo_id: Option<u32>,

    #[arg(long = "task-name", exclusive = true)]
    task_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct DeviceAuthResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: String,
    expires_in: usize,
    interval: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    token_type: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<usize>,
    scope: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let cli = Cli::parse();

    match cli.command {
        Command::Login => {
            println!("Login");
        }
        Command::Logout => {
            println!("Logout");
        }
        Command::Todos(todos_command) => match todos_command {
            TodosCommand::View(todo_view_command) => {
                println!("View Todo: {}", todo_view_command.todo_id);
            }
            TodosCommand::List => {
                println!("List Todos");
            }
            TodosCommand::Add(todo_add_command) => {
                println!("Add Todo: {}", todo_add_command.todo_name);
            }
            TodosCommand::Complete(todo_complete_command) => {
                let todo_value = match todo_complete_command.todo_id {
                    Some(todo_id) => todo_id.to_string(),
                    None => todo_complete_command.task_name.unwrap(),
                };

                println!("Complete Todo: {}", todo_value);
            }
        },
    }

    // // Get Device Code
    // let client = Client::new();
    // let resp = client
    //     .post(&format!("https://{}/oauth/device/code", config.domain))
    //     .form(&[
    //         ("client_id", config.client_id.as_str()),
    //         ("audience", config.audience.as_str()),
    //         ("scope", "openid profile email offline_access"),
    //     ])
    //     .send();

    // println!("{:#?}", resp);

    // let device_auth_response: DeviceAuthResponse = resp?.json::<DeviceAuthResponse>()?;

    // println!(
    //     "Go to {} and enter the code: {}",
    //     device_auth_response.verification_uri, device_auth_response.user_code
    // );

    // open::that(device_auth_response.verification_uri_complete)?;

    // let token_endpoint = format!("https://{}/oauth/token", config.domain);

    // // Polling for token.
    // loop {
    //     println!("Polling for token...");
    //     let resp: TokenResponse = client
    //         .post(&token_endpoint)
    //         .form(&[
    //             ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
    //             ("device_code", &device_auth_response.device_code),
    //             ("client_id", config.client_id.as_str()),
    //         ])
    //         .send()?
    //         .json::<TokenResponse>()?;

    //     println!("poll resp {:?}", resp);
    //     if let Some(access_token) = resp.access_token {
    //         println!("Access Token: {}", access_token);
    //         break;
    //     }

    //     std::thread::sleep(std::time::Duration::from_secs(
    //         device_auth_response.interval as u64,
    //     ));
    // }

    Ok(())
}
