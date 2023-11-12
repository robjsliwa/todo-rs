use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Todo {
    pub id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub task: String,
    pub completed: bool,
}

impl Todo {
    pub fn new(tenant_id: String, user_id: String, new_todo: NewTodo) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            tenant_id,
            user_id,
            task: new_todo.task,
            completed: new_todo.completed,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewTodo {
    pub task: String,
    pub completed: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateTodo {
    pub task: Option<String>,
    pub completed: Option<bool>,
}
