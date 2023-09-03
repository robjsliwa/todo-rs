use crate::error::Error;
use crate::model::todo::{NewTodo, Todo};
use async_trait::async_trait;

pub struct UserContext {
    pub tenant_id: String,
    pub user_id: String,
}

#[async_trait]
pub trait TodoStore: Send + Sync {
    async fn add_todo(&self, ctx: &UserContext, new_todo: NewTodo) -> Result<(), Error>;
    async fn get_todo(&self, ctx: &UserContext, id: String) -> Result<Option<Todo>, Error>;
    async fn get_todos(&self, ctx: &UserContext) -> Result<Vec<Todo>, Error>;
    async fn update_todo(
        &self,
        ctx: &UserContext,
        id: String,
        completed: bool,
    ) -> Result<Option<Todo>, Error>;
    async fn delete_todo(&self, ctx: &UserContext, id: String) -> Result<Option<Todo>, Error>;
}
