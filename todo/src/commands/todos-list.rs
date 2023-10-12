use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Todo {
    id: String,
    title: String,
    completed: bool,
}

#[derive(Debug, Deserialize, Default)]
struct TodoList {
    todos: Vec<Todo>,
}

pub fn todos_list(url: &str, access_token: &str) {
    println!("List command executed");
    let client = Client::new();
    let token_endpoint = format!("{}/todos", url);

    let resp = client
        .get(token_endpoint)
        .header("Authorization", format! {"Bearer {}", access_token})
        .send();

    match resp {
        Ok(response) => {
            let todos: TodoList = response.json().unwrap_or(TodoList::default());
            println!("Todos:");
            todos.todos.iter().for_each(|todo| {
                println!("{}: {} - {}", todo.id, todo.title, todo.completed);
            });
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
