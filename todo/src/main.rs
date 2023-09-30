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

#[derive(Subcommand)]
enum TodosCommand {
    View(TodoViewCommand),
    List,
    Add(TodoAddCommand),
    Complete(TodoCompleteCommand),
    Delete(TodoDeleteCommand),
}

#[derive(Parser)]
struct TodoViewCommand {
    #[arg(long = "task-id", exclusive = true)]
    todo_id: Option<String>,

    #[arg(long = "task-name", exclusive = true)]
    task_name: Option<String>,
}

#[derive(Parser)]
struct TodoAddCommand {
    #[arg(long = "task-name")]
    todo_name: String,
}

#[derive(Parser)]
struct TodoCompleteCommand {
    #[arg(long = "task-id", exclusive = true)]
    todo_id: Option<String>,

    #[arg(long = "task-name", exclusive = true)]
    task_name: Option<String>,
}

#[derive(Parser)]
struct TodoDeleteCommand {
    #[arg(long = "task-id", exclusive = true)]
    todo_id: Option<String>,

    #[arg(long = "task-name", exclusive = true)]
    task_name: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
                let todo_value = todo_view_command
                    .todo_id
                    .or_else(|| todo_view_command.task_name.clone())
                    .unwrap_or_else(|| {
                        eprintln!("You must specify either a task-id or task-name");
                        std::process::exit(1);
                    });

                println!("View Todo: {}", todo_value);
            }
            TodosCommand::List => {
                println!("List Todos");
            }
            TodosCommand::Add(todo_add_command) => {
                println!("Add Todo: {}", todo_add_command.todo_name);
            }
            TodosCommand::Complete(todo_complete_command) => {
                let todo_value = todo_complete_command
                    .todo_id
                    .or_else(|| todo_complete_command.task_name.clone())
                    .unwrap_or_else(|| {
                        eprintln!("You must specify either a task-id or task-name");
                        std::process::exit(1);
                    });

                println!("Complete Todo: {}", todo_value);
            }
            TodosCommand::Delete(todo_delete_command) => {
                let todo_value = todo_delete_command
                    .todo_id
                    .or_else(|| todo_delete_command.task_name.clone())
                    .unwrap_or_else(|| {
                        eprintln!("You must specify either a task-id or task-name");
                        std::process::exit(1);
                    });

                println!("Delete Todo: {}", todo_value);
            }
        },
    }

    Ok(())
}
