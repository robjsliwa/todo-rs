use clap::Parser;

#[derive(Parser, Debug)]
pub struct TodoAddCommand {
    #[arg(long = "task-name")]
    pub todo_name: String,
}
