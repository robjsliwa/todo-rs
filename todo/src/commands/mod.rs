#[path = "command-executor.rs"]
mod command_executor;
mod context;
mod login;
mod logout;
mod todo;
#[path = "todos-add.rs"]
mod todos_add;
#[path = "todos-add-options.rs"]
mod todos_add_options;
#[path = "todos-complete.rs"]
mod todos_complete;
#[path = "todos-delete.rs"]
mod todos_delete;
#[path = "todos-list.rs"]
mod todos_list;
#[path = "todos-options.rs"]
mod todos_options;
#[path = "todos-view.rs"]
mod todos_view;

use command_executor::CommandExecutor;
pub use context::CommandContext;
use login::login;
use logout::logout;
use todo::*;
use todos_add::todos_add;
use todos_add_options::TodoAddCommand;
use todos_complete::todos_complete;
use todos_delete::todos_delete;
use todos_list::todos_list;
use todos_options::*;
use todos_view::todos_view;

use crate::auth::get_token;
use clap::{Parser, Subcommand};

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

impl CommandExecutor for Command {
    fn execute(&self, context: &mut CommandContext) {
        match self {
            Command::Login => login(context),
            Command::Logout => logout(context),
            Command::Todos(todos_command) => todos_command.execute(context),
        }
    }
}

#[derive(Subcommand)]
enum TodosCommand {
    View(TodosSelectOptions),
    List,
    Add(TodoAddCommand),
    Complete(TodosSelectOptions),
    Delete(TodosSelectOptions),
}

impl CommandExecutor for TodosCommand {
    fn execute(&self, context: &mut CommandContext) {
        let access_token = match get_token(context) {
            Ok(token) => match token {
                Some(token) => token,
                None => {
                    eprintln!("You must login first.");
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Couldn't get credentials: {}.  Try to login again.", e);
                std::process::exit(1);
            }
        };
        match self {
            TodosCommand::View(todos_options) => {
                todos_view(todos_options, &context.config.todo_url, &access_token)
            }
            TodosCommand::List => todos_list(&context.config.todo_url, &access_token),
            TodosCommand::Add(todo_add_command) => {
                todos_add(todo_add_command, &context.config.todo_url, &access_token)
            }
            TodosCommand::Complete(todos_options) => {
                todos_complete(todos_options, &context.config.todo_url, &access_token)
            }
            TodosCommand::Delete(todos_options) => {
                todos_delete(todos_options, &context.config.todo_url, &access_token)
            }
        }
    }
}

pub fn invoke_command(context: &mut CommandContext) {
    let cli = Cli::parse();
    cli.command.execute(context);
}
