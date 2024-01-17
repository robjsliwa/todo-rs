use crate::commands::TodosSelectOptions;
use reqwest::blocking::Client;

pub fn todos_delete(options: &TodosSelectOptions, url: &str, access_token: &str) {
    let task_id = options.task_id.clone();
    let client = Client::new();
    let todo_endpoint = format!("{}/todos/{}", url, task_id);

    let resp = client
        .delete(todo_endpoint)
        .header("Authorization", format! {"Bearer {}", access_token})
        .send();

    match resp {
        Ok(_) => {
            println!("Todo deleted.");
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
