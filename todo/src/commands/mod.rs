#[path = "command-executor.rs"]
mod command_executor;
mod login;
mod logout;
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

use crate::config::Config;
use command_executor::CommandExecutor;
use login::login;
use logout::logout;
use todos_add::todos_add;
use todos_add_options::TodoAddCommand;
use todos_complete::todos_complete;
use todos_delete::todos_delete;
use todos_list::todos_list;
use todos_options::TodosOptions;
use todos_view::todos_view;

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
    fn execute(&self, config: &Config) {
        match self {
            Command::Login => login(config),
            Command::Logout => logout(),
            Command::Todos(todos_command) => todos_command.execute(config),
        }
    }
}

#[derive(Subcommand)]
enum TodosCommand {
    View(TodosOptions),
    List,
    Add(TodoAddCommand),
    Complete(TodosOptions),
    Delete(TodosOptions),
}

impl CommandExecutor for TodosCommand {
    fn execute(&self, _: &Config) {
        match self {
            TodosCommand::View(todos_options) => todos_view(todos_options),
            TodosCommand::List => todos_list(),
            TodosCommand::Add(todo_add_command) => todos_add(todo_add_command),
            TodosCommand::Complete(todos_options) => todos_complete(todos_options),
            TodosCommand::Delete(todos_options) => todos_delete(todos_options),
        }
    }
}

pub fn invoke_command(config: &Config) {
    let cli = Cli::parse();
    cli.command.execute(config);
}
