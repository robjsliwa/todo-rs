use crate::commands::todos_add_options::TodoAddCommand;

pub fn todos_add(options: &TodoAddCommand) {
    println!("Add command executed with todo_name: {:?}", options);
}
