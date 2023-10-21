use crate::error::Error;
use crate::model::{NewTodo, Todo, UpdateTodo, User};
use async_trait::async_trait;

#[derive(Debug, Clone)]
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
        update_todo: UpdateTodo,
    ) -> Result<Option<Todo>, Error>;
    async fn delete_todo(&self, ctx: &UserContext, id: String) -> Result<Option<Todo>, Error>;
    async fn create_user(
        &self,
        external_id: String,
        name: String,
        email: String,
    ) -> Result<User, Error>;
    async fn get_user(&self, external_user_id: String) -> Result<Option<User>, Error>;
}
