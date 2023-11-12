use super::Todo;
use crate::commands::TodosSelectOptions;
use reqwest::blocking::Client;

pub fn todos_view(options: &TodosSelectOptions, url: &str, access_token: &str) {
    let task_id = options.task_id.clone();
    let client = Client::new();
    let todo_endpoint = format!("{}/todos/{}", url, task_id);

    let resp = client
        .get(todo_endpoint)
        .header("Authorization", format! {"Bearer {}", access_token})
        .send();

    match resp {
        Ok(response) => {
            let todo = match response.json::<Todo>() {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return;
                }
            };

            println!("Todo:");
            println!("{}: {} - {}", todo.id, todo.task, todo.completed);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
