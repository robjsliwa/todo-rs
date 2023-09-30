use clap::Parser;

#[derive(Parser, Debug)]
pub struct TodosOptions {
    #[arg(long = "task-id", exclusive = true)]
    pub task_id: Option<String>,

    #[arg(long = "task-name", exclusive = true)]
    pub task_name: Option<String>,
}
