use super::*;
use crate::error::return_error;
use crate::storage::TodoStore;
use crate::storage::UserContext;
use std::sync::Arc;
use uuid::Uuid;
use warp::{http::Method, Filter, Rejection};

pub fn router(
    store: Arc<dyn TodoStore>,
    with_jwt: impl Filter<Extract = (UserContext,), Error = Rejection> + Clone + Send + Sync + 'static,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let with_store = warp::any().map(move || store.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["User-Agent", "Content-Type", "Authorization"])
        .allow_methods(&[Method::GET, Method::POST, Method::DELETE, Method::PATCH]);

    let get_todo_route = warp::get()
        .and(warp::path!("todos" / Uuid))
        .and(warp::path::end())
        .and(with_jwt.clone())
        .and(with_store.clone())
        .and_then(get_todo);

    let get_todos_route = warp::get()
        .and(warp::path("todos"))
        .and(warp::path::end())
        .and(with_jwt.clone())
        .and(with_store.clone())
        .and_then(get_todos);

    let add_todo_route = warp::post()
        .and(warp::path("todos"))
        .and(warp::path::end())
        .and(with_jwt.clone())
        .and(with_store.clone())
        .and(warp::body::json())
        .and_then(add_todo);

    let update_todo_route = warp::patch()
        .and(warp::path!("todos" / Uuid))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_jwt.clone())
        .and(with_store.clone())
        .and_then(update_todo);

    let delete_todo_route = warp::delete()
        .and(warp::path!("todos" / Uuid))
        .and(warp::path::end())
        .and(with_jwt)
        .and(with_store.clone())
        .and_then(delete_todo);

    get_todo_route
        .or(get_todos_route)
        .or(add_todo_route)
        .or(update_todo_route)
        .or(delete_todo_route)
        .with(cors)
        .recover(return_error)
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::model::Todo;
    use crate::storage::UserContext;
    use std::sync::Arc;
    use warp::{http::HeaderMap, reject, Filter, Rejection};

    fn with_mock_jwt(
        user_context: UserContext,
        is_valid: bool,
    ) -> impl Filter<Extract = (UserContext,), Error = Rejection> + Clone {
        warp::header::headers_cloned()
            .map(move |headers: HeaderMap| (headers.clone(), user_context.clone(), is_valid))
            .and_then(
                |(_headers, user_context, is_valid): (HeaderMap, UserContext, bool)| async move {
                    match is_valid {
                        true => Ok(user_context),
                        false => Err(reject::custom(Error::InvalidToken)),
                    }
                },
            )
    }

    #[tokio::test]
    async fn test_add_todo() {
        let store = Arc::new(crate::storage::MemStore::new("test.json".to_string()));
        let user_context = UserContext {
            tenant_id: "1".to_string(),
            user_id: "1".to_string(),
        };
        let route = super::router(store, with_mock_jwt(user_context, true));
        let resp = warp::test::request()
            .method("POST")
            .path("/todos")
            .json(&serde_json::json!({
                "task": "test task 1",
                "completed": false
            }))
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 201);
    }

    #[tokio::test]
    async fn test_get_todos() {
        let store = Arc::new(crate::storage::MemStore::new("test.json".to_string()));
        let user_context = UserContext {
            tenant_id: "1".to_string(),
            user_id: "1".to_string(),
        };
        let route = super::router(store, with_mock_jwt(user_context, true));

        let resp = warp::test::request()
            .method("POST")
            .path("/todos")
            .json(&serde_json::json!({
                "task": "test task 1",
                "completed": false
            }))
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 201);

        let resp = warp::test::request()
            .method("POST")
            .path("/todos")
            .json(&serde_json::json!({
                "task": "test task 2",
                "completed": false
            }))
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 201);

        let resp = warp::test::request()
            .method("GET")
            .path("/todos")
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 200);
        let body = resp.body();
        let todos: Vec<Todo> = serde_json::from_slice(body).unwrap();
        assert_eq!(todos.len(), 2);
    }

    #[tokio::test]
    async fn test_get_todo_not_found() {
        let store = Arc::new(crate::storage::MemStore::new("test.json".to_string()));
        let user_context = UserContext {
            tenant_id: "1".to_string(),
            user_id: "1".to_string(),
        };
        let route = super::router(store, with_mock_jwt(user_context, true));
        let resp = warp::test::request()
            .method("GET")
            .path("/todos/00000000-0000-0000-0000-000000000000")
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 404);
    }

    #[tokio::test]
    async fn test_get_todo() {
        let store = Arc::new(crate::storage::MemStore::new("test.json".to_string()));
        let user_context = UserContext {
            tenant_id: "1".to_string(),
            user_id: "1".to_string(),
        };
        let route = super::router(store, with_mock_jwt(user_context, true));

        let resp = warp::test::request()
            .method("POST")
            .path("/todos")
            .json(&serde_json::json!({
                "task": "test task 1",
                "completed": false
            }))
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 201);

        let resp = warp::test::request()
            .method("GET")
            .path("/todos")
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 200);
        let body = resp.body();
        let todos: Vec<Todo> = serde_json::from_slice(body).unwrap();
        assert_eq!(todos.len(), 1);
        let id = todos[0].id.clone();

        let resp = warp::test::request()
            .method("POST")
            .path("/todos")
            .json(&serde_json::json!({
                "task": "test task 2",
                "completed": false
            }))
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 201);

        let resp = warp::test::request()
            .method("GET")
            .path(&format!("/todos/{}", id))
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 200);
        let body = resp.body();
        let todo: Todo = serde_json::from_slice(body).unwrap();
        assert_eq!(todo.id, id);
        assert_eq!(todo.task, "test task 1");
    }

    #[tokio::test]
    async fn test_update_todo_not_found() {
        let store = Arc::new(crate::storage::MemStore::new("test.json".to_string()));
        let user_context = UserContext {
            tenant_id: "1".to_string(),
            user_id: "1".to_string(),
        };
        let route = super::router(store, with_mock_jwt(user_context, true));
        let resp = warp::test::request()
            .method("PATCH")
            .path("/todos/00000000-0000-0000-0000-000000000000")
            .json(&serde_json::json!({
                "task": "test task 1",
                "completed": false
            }))
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 404);
    }

    #[tokio::test]
    async fn test_update_todo() {
        let store = Arc::new(crate::storage::MemStore::new("test.json".to_string()));
        let user_context = UserContext {
            tenant_id: "1".to_string(),
            user_id: "1".to_string(),
        };
        let route = super::router(store, with_mock_jwt(user_context, true));

        let resp = warp::test::request()
            .method("POST")
            .path("/todos")
            .json(&serde_json::json!({
                "task": "test task 1",
                "completed": false
            }))
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 201);

        let resp = warp::test::request()
            .method("GET")
            .path("/todos")
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 200);
        let body = resp.body();
        let todos: Vec<Todo> = serde_json::from_slice(body).unwrap();
        assert_eq!(todos.len(), 1);
        let id = todos[0].id.clone();

        let resp = warp::test::request()
            .method("PATCH")
            .path(&format!("/todos/{}", id))
            .json(&serde_json::json!({
                "task": "test task 1",
                "completed": true
            }))
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 200);
        let body = resp.body();
        let todo: Todo = serde_json::from_slice(body).unwrap();
        assert_eq!(todo.id, id);
        assert_eq!(todo.task, "test task 1");
        assert!(todo.completed);
    }

    #[tokio::test]
    async fn test_delete_todo_not_found() {
        let store = Arc::new(crate::storage::MemStore::new("test.json".to_string()));
        let user_context = UserContext {
            tenant_id: "1".to_string(),
            user_id: "1".to_string(),
        };
        let route = super::router(store, with_mock_jwt(user_context, true));
        let resp = warp::test::request()
            .method("DELETE")
            .path("/todos/00000000-0000-0000-0000-000000000000")
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 404);
    }

    #[tokio::test]
    async fn test_delete_todo() {
        let store = Arc::new(crate::storage::MemStore::new("test.json".to_string()));
        let user_context = UserContext {
            tenant_id: "1".to_string(),
            user_id: "1".to_string(),
        };
        let route = super::router(store, with_mock_jwt(user_context, true));

        let resp = warp::test::request()
            .method("POST")
            .path("/todos")
            .json(&serde_json::json!({
                "task": "test task 1",
                "completed": false
            }))
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 201);

        let resp = warp::test::request()
            .method("GET")
            .path("/todos")
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 200);
        let body = resp.body();
        let todos: Vec<Todo> = serde_json::from_slice(body).unwrap();
        assert_eq!(todos.len(), 1);
        let id = todos[0].id.clone();

        let resp = warp::test::request()
            .method("DELETE")
            .path(&format!("/todos/{}", id))
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 204);

        let resp = warp::test::request()
            .method("GET")
            .path("/todos")
            .reply(&route)
            .await;
        assert_eq!(resp.status(), 200);
        let body = resp.body();
        let todos: Vec<Todo> = serde_json::from_slice(body).unwrap();
        assert_eq!(todos.len(), 0);
    }
}
