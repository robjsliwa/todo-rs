use crate::error::Error;
use crate::model::todo::{NewTodo, Todo};
use crate::storage::store::{TodoStore, UserContext};
use async_trait::async_trait;
use std::collections::HashMap;
use std::process;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct MemStore {
    pub objects: Arc<RwLock<HashMap<String, Todo>>>,
    file_path: String,
}

impl MemStore {
    pub fn new(file_path: String) -> Self {
        MemStore {
            objects: Arc::new(RwLock::new(Self::load(&file_path))),
            file_path,
        }
    }

    fn load(file_path: &str) -> HashMap<String, Todo> {
        match std::fs::read_to_string(file_path) {
            Ok(file) => serde_json::from_str(&file).unwrap_or_else(|_| {
                eprintln!("Failed to parse the JSON. Exiting...");
                process::exit(1);
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // File not found, continue
                HashMap::new()
            }
            Err(e) => {
                eprintln!("An error occurred while reading the file: {}...", e);
                process::exit(1);
            }
        }
    }

    pub async fn shutdown(&self) -> std::io::Result<()> {
        let data = self.objects.read().await;
        let json = serde_json::to_string(&*data).expect("Failed to save data!");
        tokio::fs::write(&self.file_path, json).await
    }
}

#[async_trait]
impl TodoStore for MemStore {
    async fn add_todo(&self, ctx: &UserContext, new_todo: NewTodo) -> Result<(), Error> {
        let mut data = self.objects.write().await;
        let todo = Todo::new(ctx.tenant_id.clone(), ctx.user_id.clone(), new_todo);
        data.insert(todo.id.clone(), todo);
        Ok(())
    }

    async fn get_todo(&self, ctx: &UserContext, id: String) -> Result<Option<Todo>, Error> {
        let data = self.objects.read().await;
        let todo = data.get(&id).cloned();
        if todo.is_some_and(|t| t.user_id != ctx.user_id || t.tenant_id != ctx.tenant_id) {
            return Err(Error::Unauthorized);
        }
        Ok(data.get(&id).cloned())
    }

    async fn get_todos(&self, ctx: &UserContext) -> Result<Vec<Todo>, Error> {
        let data = self.objects.read().await;
        let filtered_todos = data
            .values()
            .filter(|todo| todo.tenant_id == ctx.tenant_id && todo.user_id == ctx.user_id)
            .cloned()
            .collect::<Vec<Todo>>();
        Ok(filtered_todos)
    }

    async fn update_todo(
        &self,
        ctx: &UserContext,
        id: String,
        completed: bool,
    ) -> Result<Option<Todo>, Error> {
        let mut data = self.objects.write().await;
        if let Some(todo) = data.get_mut(&id) {
            if todo.user_id != ctx.user_id || todo.tenant_id != ctx.tenant_id {
                return Err(Error::Unauthorized);
            }
            todo.completed = completed;
            Ok(Some(todo.clone()))
        } else {
            Ok(None)
        }
    }

    async fn delete_todo(&self, ctx: &UserContext, id: String) -> Result<Option<Todo>, Error> {
        let mut data = self.objects.write().await;
        if let Some(todo) = data.get(&id) {
            if todo.tenant_id == ctx.tenant_id && todo.user_id == ctx.user_id {
                return Ok(data.remove(&id));
            }
        }
        Ok(None)
    }
}
