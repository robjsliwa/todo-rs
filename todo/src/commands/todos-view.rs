use crate::commands::todos_options::TodosOptions;

pub fn todos_view(options: &TodosOptions) {
    let todo_value = options
        .task_id
        .as_ref()
        .or(options.task_name.as_ref())
        .unwrap_or_else(|| {
            eprintln!("You must specify either a task-id or task-name");
            std::process::exit(1);
        });
    println!("View command: {:?}", todo_value);
}
