use super::Todo;
use reqwest::blocking::Client;

pub fn todos_list(url: &str, access_token: &str) {
    let client = Client::new();
    let todo_endpoint = format!("{}/todos", url);

    let resp = client
        .get(todo_endpoint)
        .header("Authorization", format! {"Bearer {}", access_token})
        .send();

    match resp {
        Ok(response) => {
            let todos = match response.json::<Vec<Todo>>() {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return;
                }
            };
            if todos.is_empty() {
                println!("No todos found.");
                return;
            }
            println!("Todos:");
            todos.iter().for_each(|todo| {
                println!("{}: {} - {}", todo.id, todo.task, todo.completed);
            });
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
