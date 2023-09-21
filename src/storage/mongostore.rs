use crate::error::Error;
use crate::model::todo::{NewTodo, Todo, UpdateTodo};
use crate::storage::store::{TodoStore, UserContext};
use async_trait::async_trait;
use futures::stream::TryStreamExt;
use log::{error, info};
use mongodb::bson::{doc, Document};
use mongodb::{Client, Collection};

const DB_NAME: &str = "todo";

macro_rules! update_todo {
    ($updatetodo:expr) => {{
        let mut doc = Document::new();

        if let Some(ref task) = $updatetodo.task {
            doc.insert("task", task);
        }

        if let Some(ref completed) = $updatetodo.completed {
            doc.insert("completed", completed);
        }

        doc
    }};
}

async fn mongo_result<T>(
    result: Result<Option<T>, mongodb::error::Error>,
    operation: &str,
) -> Result<Option<T>, Error> {
    match result {
        Ok(None) => Err(Error::NotFound),
        Ok(Some(item)) => Ok(Some(item)),
        Err(e) => {
            error!("Failed to {}: {:?}", operation, e);
            Err(Error::DatabaseOperationFailed(format!(
                "Failed to {}: {:?}",
                operation, e
            )))
        }
    }
}

#[derive(Debug, Clone)]
pub struct MongoStore {
    todo_col: Collection<Todo>,
}

impl MongoStore {
    pub async fn init(mongo_uri: String) -> Result<Self, Box<dyn std::error::Error>> {
        let todo_col: Collection<Todo> = Self::connect(mongo_uri).await?;
        Ok(Self { todo_col })
    }

    async fn connect(mongo_uri: String) -> Result<Collection<Todo>, Box<dyn std::error::Error>> {
        let client = Client::with_uri_str(mongo_uri).await?;
        let db = client.database(DB_NAME);
        let col: Collection<Todo> = db.collection("Todos");
        Ok(col)
    }
}

#[async_trait]
impl TodoStore for MongoStore {
    async fn add_todo(&self, ctx: &UserContext, new_todo: NewTodo) -> Result<(), Error> {
        let todo = Todo::new(ctx.tenant_id.clone(), ctx.user_id.clone(), new_todo);
        self.todo_col
            .insert_one(todo.clone(), None)
            .await
            .map_err(|e| {
                error!("Failed to insert todo: {:?}", e);
                Error::DatabaseOperationFailed(format!("Failed to insert todo: {:?}", e))
            })?;
        info!("Added todo: {:?}", todo);
        Ok(())
    }

    async fn get_todo(&self, ctx: &UserContext, id: String) -> Result<Option<Todo>, Error> {
        let filter = doc! {
            "id": id,
            "tenant_id": ctx.tenant_id.clone(),
            "user_id": ctx.user_id.clone(),
        };
        let result = self.todo_col.find_one(filter, None).await;
        mongo_result(result, "get todo").await
    }

    async fn get_todos(&self, ctx: &UserContext) -> Result<Vec<Todo>, Error> {
        let filter = doc! {
            "tenant_id": ctx.tenant_id.clone(),
            "user_id": ctx.user_id.clone(),
        };
        let cursor = self.todo_col.find(filter, None).await.map_err(|e| {
            error!("Failed create cursor to get todos: {:?}", e);
            Error::DatabaseOperationFailed(format!("Failed create cursor to get todos: {:?}", e))
        })?;
        let todos: Vec<Todo> = cursor.try_collect().await.map_err(|e| {
            error!("Failed to get todos: {:?}", e);
            Error::DatabaseOperationFailed(format!("Failed to get todos: {:?}", e))
        })?;
        Ok(todos)
    }

    async fn update_todo(
        &self,
        ctx: &UserContext,
        id: String,
        update_todo: UpdateTodo,
    ) -> Result<Option<Todo>, Error> {
        let filter = doc! {
            "id": id,
            "tenant_id": ctx.tenant_id.clone(),
            "user_id": ctx.user_id.clone(),
        };
        let update = doc! {
            "$set": update_todo!(update_todo),
        };
        let result = self
            .todo_col
            .find_one_and_update(filter, update, None)
            .await;
        mongo_result(result, "update todo").await
    }

    async fn delete_todo(&self, ctx: &UserContext, id: String) -> Result<Option<Todo>, Error> {
        let filter = doc! {
            "id": id,
            "tenant_id": ctx.tenant_id.clone(),
            "user_id": ctx.user_id.clone(),
        };
        let result = self.todo_col.find_one_and_delete(filter, None).await;
        mongo_result(result, "delete todo").await
    }
}
