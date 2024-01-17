use super::Todo;
use crate::commands::TodosSelectOptions;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTodo {
    pub completed: bool,
}

pub fn todos_complete(options: &TodosSelectOptions, url: &str, access_token: &str) {
    let task_id = options.task_id.clone();
    let client = Client::new();
    let todo_endpoint = format!("{}/todos/{}", url, task_id);
    let update_todo = UpdateTodo { completed: true };

    let resp = client
        .patch(todo_endpoint)
        .header("Authorization", format! {"Bearer {}", access_token})
        .json(&update_todo)
        .send();

    match resp {
        Ok(response) => {
            let _ = match response.json::<Todo>() {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return;
                }
            };

            println!("Todo completed.");
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
