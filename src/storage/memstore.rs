use crate::error::Error;
use crate::model::todo::{NewTodo, Todo, UpdateTodo};
use crate::storage::store::{TodoStore, UserContext};
use async_trait::async_trait;
use std::collections::HashMap;
use std::process;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct MemStore {
    pub objects: Arc<RwLock<HashMap<String, Todo>>>,
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
        if let Some(todo) = data.get(&id) {
            if todo.user_id != ctx.user_id || todo.tenant_id != ctx.tenant_id {
                return Err(Error::Unauthorized);
            }
            return Ok(Some(todo.clone()));
        }
        Err(Error::NotFound)
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
        update_todo: UpdateTodo,
    ) -> Result<Option<Todo>, Error> {
        let mut data = self.objects.write().await;
        if let Some(todo) = data.get_mut(&id) {
            if todo.user_id != ctx.user_id || todo.tenant_id != ctx.tenant_id {
                return Err(Error::Unauthorized);
            }
            todo.completed = match update_todo.completed {
                Some(completed) => completed,
                None => todo.completed,
            };
            todo.task = match update_todo.task {
                Some(task) => task,
                None => todo.task.clone(),
            };
            Ok(Some(todo.clone()))
        } else {
            Err(Error::NotFound)
        }
    }

    async fn delete_todo(&self, ctx: &UserContext, id: String) -> Result<Option<Todo>, Error> {
        let mut data = self.objects.write().await;
        if let Some(todo) = data.get(&id) {
            if todo.tenant_id == ctx.tenant_id && todo.user_id == ctx.user_id {
                return Ok(data.remove(&id));
            }
        }
        Err(Error::NotFound)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_add_todo() {
        use super::*;
        let store = MemStore::new("test.json".to_string());
        let ctx = UserContext {
            tenant_id: "tenant".to_string(),
            user_id: "user".to_string(),
        };
        let new_todo = NewTodo {
            task: "test".to_string(),
            completed: false,
        };
        store.add_todo(&ctx, new_todo).await.unwrap();
        let todos = store.get_todos(&ctx).await.unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].task, "test");
        assert!(!todos[0].completed);
        assert_eq!(todos[0].user_id, "user");
        assert_eq!(todos[0].tenant_id, "tenant");
    }

    #[tokio::test]
    async fn test_get_todo() {
        use super::*;
        let store = MemStore::new("test.json".to_string());
        let ctx = UserContext {
            tenant_id: "tenant".to_string(),
            user_id: "user".to_string(),
        };
        let new_todo = NewTodo {
            task: "test".to_string(),
            completed: false,
        };
        store.add_todo(&ctx, new_todo).await.unwrap();
        let todos = store.get_todos(&ctx).await.unwrap();
        assert_eq!(todos.len(), 1);
        let todo = store.get_todo(&ctx, todos[0].id.clone()).await.unwrap();
        assert_eq!(todo.as_ref().unwrap().task, "test");
        assert!(!todo.as_ref().unwrap().completed);
        assert_eq!(todo.as_ref().unwrap().user_id, "user");
        assert_eq!(todo.as_ref().unwrap().tenant_id, "tenant");
    }

    #[tokio::test]
    async fn test_get_todos() {
        use super::*;
        let store = MemStore::new("test.json".to_string());
        let ctx = UserContext {
            tenant_id: "tenant".to_string(),
            user_id: "user".to_string(),
        };
        let new_todo = NewTodo {
            task: "test".to_string(),
            completed: false,
        };
        store.add_todo(&ctx, new_todo).await.unwrap();
        let ctx2 = UserContext {
            tenant_id: "tenant".to_string(),
            user_id: "user2".to_string(),
        };
        let new_todo2 = NewTodo {
            task: "test2".to_string(),
            completed: false,
        };
        store.add_todo(&ctx2, new_todo2).await.unwrap();
        let todos = store.get_todos(&ctx).await.unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].task, "test");
        assert!(!todos[0].completed);
        assert_eq!(todos[0].user_id, "user");
        assert_eq!(todos[0].tenant_id, "tenant");
        let todos2 = store.get_todos(&ctx2).await.unwrap();
        assert_eq!(todos2.len(), 1);
        assert_eq!(todos2[0].task, "test2");
        assert!(!todos2[0].completed);
        assert_eq!(todos2[0].user_id, "user2");
        assert_eq!(todos2[0].tenant_id, "tenant");
    }

    #[tokio::test]
    async fn test_update_todo() {
        use super::*;
        let store = MemStore::new("test.json".to_string());
        let ctx = UserContext {
            tenant_id: "tenant".to_string(),
            user_id: "user".to_string(),
        };
        let new_todo = NewTodo {
            task: "test".to_string(),
            completed: false,
        };
        store.add_todo(&ctx, new_todo).await.unwrap();
        let todos = store.get_todos(&ctx).await.unwrap();
        assert_eq!(todos.len(), 1);
        let update_todo = UpdateTodo {
            task: Some("test2".to_string()),
            completed: Some(true),
        };
        let todo = store
            .update_todo(&ctx, todos[0].id.clone(), update_todo)
            .await
            .unwrap();
        assert_eq!(todo.as_ref().unwrap().task, "test2");
        assert!(todo.as_ref().unwrap().completed);
        assert_eq!(todo.as_ref().unwrap().user_id, "user");
        assert_eq!(todo.as_ref().unwrap().tenant_id, "tenant");
    }

    #[tokio::test]
    async fn test_delete_todo() {
        use super::*;
        let store = MemStore::new("test.json".to_string());
        let ctx = UserContext {
            tenant_id: "tenant".to_string(),
            user_id: "user".to_string(),
        };
        let new_todo = NewTodo {
            task: "test".to_string(),
            completed: false,
        };
        store.add_todo(&ctx, new_todo).await.unwrap();
        let todos = store.get_todos(&ctx).await.unwrap();
        assert_eq!(todos.len(), 1);
        let todo = store.delete_todo(&ctx, todos[0].id.clone()).await.unwrap();
        assert_eq!(todo.as_ref().unwrap().task, "test");
        assert!(!todo.as_ref().unwrap().completed);
        assert_eq!(todo.as_ref().unwrap().user_id, "user");
        assert_eq!(todo.as_ref().unwrap().tenant_id, "tenant");
        let todos = store.get_todos(&ctx).await.unwrap();
        assert_eq!(todos.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_todo_not_found() {
        use super::*;
        let store = MemStore::new("test.json".to_string());
        let ctx = UserContext {
            tenant_id: "tenant".to_string(),
            user_id: "user".to_string(),
        };
        let new_todo = NewTodo {
            task: "test".to_string(),
            completed: false,
        };
        store.add_todo(&ctx, new_todo).await.unwrap();
        let todos = store.get_todos(&ctx).await.unwrap();
        assert_eq!(todos.len(), 1);
        let ctx2 = UserContext {
            tenant_id: "tenant".to_string(),
            user_id: "user2".to_string(),
        };
        let expected_result = store.delete_todo(&ctx2, todos[0].id.clone()).await;
        assert_eq!(expected_result, Err(Error::NotFound));
        let todos = store.get_todos(&ctx).await.unwrap();
        assert_eq!(todos.len(), 1);
    }

    #[tokio::test]
    async fn test_update_todo_unauthorized() {
        use super::*;
        let store = MemStore::new("test.json".to_string());
        let ctx = UserContext {
            tenant_id: "tenant".to_string(),
            user_id: "user".to_string(),
        };
        let new_todo = NewTodo {
            task: "test".to_string(),
            completed: false,
        };
        store.add_todo(&ctx, new_todo).await.unwrap();
        let todos = store.get_todos(&ctx).await.unwrap();
        assert_eq!(todos.len(), 1);
        let ctx2 = UserContext {
            tenant_id: "tenant".to_string(),
            user_id: "user2".to_string(),
        };
        let update_todo = UpdateTodo {
            task: Some("test2".to_string()),
            completed: Some(true),
        };
        let expected_result = store
            .update_todo(&ctx2, todos[0].id.clone(), update_todo)
            .await;
        assert_eq!(expected_result, Err(Error::Unauthorized));
        let todos = store.get_todos(&ctx).await.unwrap();
        assert_eq!(todos.len(), 1);
    }

    #[tokio::test]
    async fn test_get_todo_not_found() {
        use super::*;
        let store = MemStore::new("test.json".to_string());
        let ctx = UserContext {
            tenant_id: "tenant".to_string(),
            user_id: "user".to_string(),
        };
        let new_todo = NewTodo {
            task: "test".to_string(),
            completed: false,
        };
        store.add_todo(&ctx, new_todo).await.unwrap();
        let ctx2 = UserContext {
            tenant_id: "tenant".to_string(),
            user_id: "user2".to_string(),
        };
        let expected_result = store.get_todo(&ctx2, "test".to_string()).await;
        assert_eq!(expected_result, Err(Error::NotFound));
    }

    #[tokio::test]
    async fn test_get_todos_not_found() {
        use super::*;
        let store = MemStore::new("test.json".to_string());
        let ctx2 = UserContext {
            tenant_id: "tenant".to_string(),
            user_id: "user2".to_string(),
        };
        let todos = store.get_todos(&ctx2).await.unwrap();
        assert_eq!(todos.len(), 0);
    }
}
