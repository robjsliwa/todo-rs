use crate::commands::todos_add_options::TodoAddCommand;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewTodo {
    pub task: String,
    pub completed: bool,
}

pub fn todos_add(options: &TodoAddCommand, url: &str, access_token: &str) {
    let new_todo = NewTodo {
        task: options.todo_name.clone(),
        completed: false,
    };
    let client = Client::new();
    let todo_endpoint = format!("{}/todos", url);

    let resp = client
        .post(todo_endpoint)
        .header("Authorization", format! {"Bearer {}", access_token})
        .json(&new_todo)
        .send();

    match resp {
        Ok(_) => {
            println!("Todo added");
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
