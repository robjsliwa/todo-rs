# Building a Multi-Tenant Todo Server in Rust: Part 2

## Introduction

In Part 1 of this series, we began our journey into building a multi-tenant Todo server using Rust. We set up the basic project structure, implemented Warp routes, and even provided an in-memory storage solution. In this second installment, we're going to extend our application by:

- Reorganizing our codebase into a modular folder structure
- Defining the Todo model with multi-tenancy in mind
- Implementing RESTful APIs for our Todo server
- Protecting access to our APIs with authentication
- Introducing a hexagonal architecture for our storage layer

So, let's get started!

## Multi-Tenancy

Before we begin, let's take a moment to discuss multi-tenancy. Multi-tenancy is a software architecture pattern where a single instance of software serves multiple tenants. A tenant is a group of users who share a common access with specific privileges to the resources. With multi-tenancy, a software instance is shared by multiple tenants, but each tenant's data is isolated and remains invisible to other tenants. Multi-tenancy is a popular choice for SaaS applications because it allows for a single instance of the software to be shared by multiple customers, reducing operational costs.

We will need to consider multi-tenancy at two levels. One at the API server level and the second one at the database level. There are couple archtictural patterns that we can use to achieve multi-tenancy.

### Shared Database, Shared Schema

This approach involves all tenants sharing the same database and the same schema. Each table includes a tenant ID field to differentiate rows belonging to different tenants. This is the simplest model but carries the risk of bugs or security holes potentially exposing data from one tenant to another.

```mermaid
graph LR
A[Tenant 1] --> B[Shared DB & Schema]
C[Tenant 2] --> B
D[Tenant 3] --> B
```

### Shared Database, Separate Schema

In this pattern, each tenant has its own schema within the shared database. This provides better isolation than the shared schema method but there might still be performance issues as the load increases.

```mermaid
graph LR
A[Tenant 1] --> B[Shared DB]
A --> D[Schema 1]
C[Tenant 2] --> B
C --> E[Schema 2]
F[Tenant 3] --> B
F --> G[Schema 3]
```

### Separate Database

Each tenant has its own database. This provides the best isolation and can be the best for performance, but is the most complex and costly solution.

```mermaid
graph LR
A[Tenant 1] --> B[Database 1]
C[Tenant 2] --> D[Database 2]
E[Tenant 3] --> F[Database 3]
```

### Our Approach

At the API server level, we will ensure that each user accesses resources that belong to them for the tenant that they are logged in by using JWT tokens with custom claims that store user id and tenant id. Server will be able to use these two values to ensure that the user has access only to the resources they are authorized for.

At the database level we will use single database, single schema approach. What that means from perspective of schema design is that we will have to add tenant id and user id to each table. This will allow us to ensure that each user can only access resources that belong to them.

## Authentication

Let's begin by adding authentication to our Todo server. We'll be using [JSON Web Tokens](https://jwt.io/) (JWT) to authenticate users. JWTs are a popular choice for authentication because they are stateless, meaning that the server doesn't need to store any session information. This makes JWTs a great choice for our multi-tenant Todo server because we don't need to worry about storing session information for each tenant.

To keep simple for now we will create JWT tokens ourselves for now. In future articles we will extend the project to use a third party authentication provider like Auth0.

### Creating JWT Tokens

This is an opportunity to create simple command line tool that will accept user id and tenant id and generate JWT token for us. We will use [jsonwebtoken](https://crates.io/crates/jsonwebtoken) crate to generate JWT tokens.

In addition, to the above parameters we need to provide token duration. Duration will be in seconds. We will use 1 hour as defualt duration.

Final parameter is the secret that will be used to sign the token. When the server receives the token it will use the same secret to verify the token.

To implement command line utility let's create new binary. Execute the following command from project's root folder:

```bash
cargo new --bin genjwt
```

This will create new folder called `genjwt` with `main.rs` file inside. Let's update `main.rs` file to accept user id, tenant id, duration and secret as command line arguments. For handling command line arguments we will use [clap](https://crates.io/crates/clap) crate.

Add following to the genjwt/Cargo.toml file:

```toml
[dependencies]
jsonwebtoken = "8.3.0"
chrono = "0.4.28"
clap = { version = "4.4.2", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

```rust
use chrono::prelude::*;
use clap::Parser;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    secret: String,
    #[arg(short, long)]
    tenant_id: String,
    #[arg(short, long)]
    user_id: String,
    #[arg(short, long, default_value_t = 3600)]
    exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    tenant_id: String,
    user_id: String,
    exp: usize,
}

fn main() {
    let args = Args::parse();
    let tenant_id = args.tenant_id;
    let user_id = args.user_id;
    let duration = args.exp;

    let my_claims = Claims {
        tenant_id,
        user_id,
        exp: (Utc::now() + chrono::Duration::seconds(duration as i64)).timestamp() as usize,
    };

    let key = args.secret;
    let token = match encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(key.as_ref()),
    ) {
        Ok(t) => t,
        Err(_) => panic!("Error generating the token"),
    };
    println!("Generated JWT: {}", token);
}
```

Let's build the project and test it out:

```bash
cargo build
```

```bash
cargo run -- --secret secret --tenant-id 1 --user-id 1
```

This should generate JWT token that looks like this:

```bash
Generated JWT: eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ0ZW5hbnRfaWQiOiIxIiwidXNlcl9pZCI6IjEiLCJleHAiOjE2OTM1ODgwNDV9.Hc8u2POi18YZOXmBOH09_rfo8PPyCKawWVviHFy7PJQ
```

We can use jwt.io to decode the token and verify that it contains the correct information:

![JWT Token](./images/jwt.png)

If you type in the secret that we used to generate the token you should see that token is verified successfully:

![JWT Token](./images/jwt-decoded.png)

Now we are ready to start building Todo APIs and protect them with our JWT tokens.

## Setting up the Project Structure

As our project grows, it's essential to keep our codebase organized. To achieve this, we'll create separate folders for our routes, models, and storage logic. Create the following folders in your project:

- routes/
- model/
- storage/

Your directory structure should now look something like this:

```bash
src/
├── routes/
├── model/
├── storage/
├── main.rs
└── ...
```

`routes` will contain all of our route handlers. `model` will contain our data models. `storage` will contain our storage logic.

## Creating the Todo Model

To make our application multi-tenant, we'll need to update our `Object` struct to a more meaningful `Todo` struct. The `Todo` struct will have the following fields:

- id: A unique identifier for each todo item
- tenant_id: Identifies the tenant to which this todo item belongs
- user_id: Identifies the user within the tenant
- task: The actual task description
- completed: Whether the task is completed or not

Create a new file named todo.rs inside the model/ folder and add the following code:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub task: String,
    pub completed: bool,
}
```

## Decoupling Storage with Hexagonal Architecture

### Motivation

We've been building our Todo server using an in-memory storage so far. While this is great for prototyping, it's obviously not suitable for production. In the next article we will switch to using a database but right now we want to focus on decoupling our storage logic from the rest of the application so that we can easily switch between different storage implementations. This is where [Hexagonal Architecture](<https://en.wikipedia.org/wiki/Hexagonal_architecture_(software)>) comes in.

### What is Hexagonal Architecture?

Hexagonal Architecture, also known as Ports and Adapters, allows us to decouple the core logic of our application from external concerns like storage, UI, and others. The main idea is to define clear contracts or interfaces that the core logic expects, and then implement these interfaces for each external concern (like storage).

### Benefits of Hexagonal Architecture

- **Ease of Testing**: By abstracting external concerns, we can easily mock them during testing.
- **Flexibility**: We can easily replace one storage solution with another with minimal code changes.
- **Separation of Concerns**: It helps to keep the core logic isolated, making the codebase easier to understand and maintain.

### Our Implementation

Let's start by defining contract/interface first. In `storage` folder create new file called `store.rs` and add the following code:

```rust
use crate::error::Error;
use crate::model::todo::{NewTodo, Todo};
use async_trait::async_trait;

pub struct UserContext {
    pub tenant_id: String,
    pub user_id: String,
}

#[async_trait]
pub trait TodoStore {
    async fn add_todo(&self, ctx: &UserContext, new_todo: NewTodo) -> Result<(), Error>;
    async fn get_todo(&self, ctx: &UserContext, id: String) -> Result<Option<Todo>, Error>;
    async fn get_todos(&self, ctx: &UserContext) -> Result<Vec<Todo>, Error>;
    async fn update_todo(
        &self,
        ctx: &UserContext,
        id: String,
        completed: bool,
    ) -> Result<Option<Todo>, Error>;
    async fn delete_todo(
        &self,
        ctx: &UserContext,
        id: String,
    ) -> Result<Option<Todo>, Error>;
}
```

Our interface is quite simple and defines the following methods:

- `add_todo`: Adds a new todo item
- 'get_todo`: Gets a todo item by id
- `get_todos`: Gets all todo items
- `update_todo`: Updates the completed status of a todo item
- `delete_todo`: Deletes a todo item

Notice that for all these functions we are passing a `UserContext` object. This object contains the `tenant_id` and `user_id` of the user making the request. This is how we will implement multi-tenancy. We will use these values to filter the todo items by tenant and user. We will get those values from the JWT token that the user will send with each request.

Also for `add_todo` we need to provide `task` and `completed`. We could pass this in as two parameters but if we were to ever extend todo to hold some more information the list of parameters will grow. Let's just define `NewTodo` struct. In `model\todo.rs` add following definition:

```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct NewTodo {
    pub task: String,
    pub completed: bool,
}
```

We don't want to use `Todo` struct as we want server to define `id` rather then client.

Notice that to implement our interface we used Rust's trait.

### What is a Trait in Rust?

In Rust, a trait is a way to group method signatures together to define a set of behaviors necessary for a particular purpose. Traits are similar to interfaces in languages like Java and C#. They allow us to write code that is agnostic to the specific types, as long as these types implement the methods defined in the trait.

### Why `async_trait`?

Rust's native trait system doesn't support asynchronous methods directly yet. To work around this, we use the `async_trait` crate that provides a procedural macro to enable async functions in traits. This allows us to define our storage operations as asynchronous, which is essential for IO-bound tasks like database operations.

In order to use `async_trait` we need to add it to our `Cargo.toml` file:

```toml
[dependencies]
.
.
.
async-trait = "0.1.73"
```

### Implementing the Storage Interface

We've defined a trait `TodoStore` in store.rs that serves as the contract for our storage solutions. Now we need to implement this trait for our in-memory storage solution.

First move `store.rs` to `store\memstore.rs` and add implementation of `TodoStore` trait:

```rust
#[async_trait]
impl TodoStore for MemStore {
    async fn add_todo(&self, ctx: &UserContext, new_todo: NewTodo) -> Result<(), Error> {
        ...
    }

    async fn get_todo(&self, ctx: &UserContext, id: String) -> Result<Option<Todo>, Error> {
        ...
    }

    async fn get_todos(&self, ctx: &UserContext) -> Result<Vec<Todo>, Error> {
        ...
    }

    async fn update_todo(
        &self,
        ctx: &UserContext,
        id: String,
        completed: bool,
    ) -> Result<Option<Todo>, Error> {
        ...
    }

    async fn delete_todo(
        &self,
        ctx: &UserContext,
        id: String,
    ) -> Result<Option<Todo>, Error> {
        ...
    }
}
```

Here we took our old Store object and renamed it to MemStore and kept its implementation the same as before:

```rust
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
```

But then we added implementation of TodoStore trait. Let's go over each method.

#### Adding Todo Items

The `add_todo` method is straightforward. We acquire a write lock on the in-memory data store, which is a HashMap, and insert the new Todo item. Note how we construct a new Todo object that includes the tenant_id and user_id from the UserContext.

```rust
    async fn add_todo(&self, ctx: &UserContext, new_todo: NewTodo) -> Result<(), Error> {
        let mut data = self.objects.write().await;
        let todo = Todo::new(ctx.tenant_id.clone(), ctx.user_id.clone(), new_todo);
        data.insert(todo.id.clone(), todo);
        Ok(())
    }
```

#### Fetching Todo Items

The `get_todo` and `get_todos` methods read from the data store, but they also filter results based on the tenant_id and user_id. This ensures that users only have access to their own data within their own tenant.

```rust
    async fn get_todo(&self, ctx: &UserContext, id: String) -> Result<Option<Todo>, Error> {
        let data = self.objects.read().await;
        let todo = data.get(&id).cloned();
        if todo.is_some_and(|t| t.user_id != ctx.user_id || t.tenant_id != ctx.tenant_id) {
            return Err(Error::Unauthorized);
        }
        Ok(data.get(&id).cloned())
    }
```

and

```rust
    async fn get_todos(&self, ctx: &UserContext) -> Result<Vec<Todo>, Error> {
        let data = self.objects.read().await;
        let filtered_todos = data
            .values()
            .filter(|todo| todo.tenant_id == ctx.tenant_id && todo.user_id == ctx.user_id)
            .cloned()
            .collect::<Vec<Todo>>();
        Ok(filtered_todos)
    }
```

#### Updating and Deleting Todo Items

Similar to fetching, the `update_todo` and `delete_todo` methods also check for user and tenant ownership before modifying or removing any Todo items.

```rust
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
```

and

```rust
    async fn delete_todo(&self, ctx: &UserContext, id: String) -> Result<Option<Todo>, Error> {
        let mut data = self.objects.write().await;
        if let Some(todo) = data.get(&id) {
            if todo.tenant_id == ctx.tenant_id && todo.user_id == ctx.user_id {
                return Ok(data.remove(&id));
            }
        }
        Ok(None)
    }
```

The above implementation showcases the power of Hexagonal Architecture. We've managed to encapsulate all storage-related operations within this MemStore class, which implements the TodoStore trait. This decoupling makes it easier to replace or extend our storage solutions in the future.

## Implementing the Todo Service

We have all the parts needed to implement APIs for our Todo server. Let's start by implementing add todo functionality so create a new folder `routes` and add file called `add_todo.rs` inside of that folder.

```rust
use crate::model::todo::NewTodo;
use crate::storage::store::{TodoStore, UserContext};
use std::sync::Arc;
use warp::http::StatusCode;

pub async fn add_todo(
    user: UserContext,
    store: Arc<dyn TodoStore>,
    new_todo: NewTodo,
) -> Result<impl warp::Reply, warp::Rejection> {
    store.add_todo(&user, new_todo).await?;
    Ok(StatusCode::CREATED)
}
```

The implementation is pretty straightforward as we are just calling `add_todo` method on our storage object. We are going to get `new_todo` from the request body and `user` from the JWT token.

Notice that we are using `Arc<dyn TodoStore>` as the type for our storage object. We need to do this because Warp functions are executed on different threads. Warp also wants to know the size of the parameters we are passing in. In order for this to work we need to add `Send` and `Sync` traits to our TodoStore trait. Let's update `store.rs` file:

```rust
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
}
```

The Send and Sync traits in Rust are marker traits that control the ability of types to be transferred or accessed across thread boundaries, respectively.

**Send**

The `Send` trait indicates that ownership of a type implementing this trait can be transferred safely between threads. In other words, if a type T implements Send, then it is safe to move T from one thread to another.

For example, basic types like i32, String, and collections like Vec<T> and HashMap<K, V> where T, K, and V are Send, are themselves Send.

However, types that include raw pointers or references to thread-local data are typically not Send.

**Sync**

The `Sync` trait indicates that it is safe to access a type T from multiple threads concurrently. If a type T is Sync, then &T (a reference to T) can safely be shared between multiple threads.

For example, if you have an Arc<T> where T: Sync, multiple threads can read from T safely.

We also need to initialize our storage object with `MemStore` in `main.rs` file:

```rust
    let mem_store = MemStore::new("./data.json".to_string());
    let store: Arc<dyn TodoStore> = Arc::new(mem_store.clone());
    let store_for_routes = store.clone();
```

At this point we have everything that we need to create and call `add_todo` route except user context. We will get it from JWT passed in Authorization header in the format:

```
Authorization: Bearer <JWT>
```

Let's create new folder `auth` and add file `token_from_header.rs`:

```rust
use crate::error::Error;
use warp::http::header::{HeaderMap, HeaderValue, AUTHORIZATION};

pub fn token_from_header(headers: &HeaderMap<HeaderValue>) -> Result<String, Error> {
    const BEARER: &str = "Bearer ";
    let header = match headers.get(AUTHORIZATION) {
        Some(v) => v,
        None => return Err(Error::Unauthorized),
    };
    let auth_header = match std::str::from_utf8(header.as_bytes()) {
        Ok(v) => v,
        Err(_) => return Err(Error::Unauthorized),
    };
    if !auth_header.starts_with(BEARER) {
        return Err(Error::Unauthorized);
    }
    Ok(auth_header.trim_start_matches(BEARER).to_owned())
}
```

The purpose of this function is to verify that Authorization header is present and that it starts with `Bearer ` and then return the token.

Now how do we verify this JWT and pass it to the API handler? To do that we need Warp filter. Let's put it in its own file in `auth` folder called `with_jwt.rs`:

```rust
use warp::{
    http::HeaderMap,
    reject,
    Filter, Rejection,
};
use crate::auth::token_from_header::token_from_header;
use crate::error::Error;
use crate::storage::store::UserContext;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    tenant_id: String,
    user_id: String,
    exp: usize,
}

pub fn with_jwt(jwt_secret: String) -> impl Filter<Extract = (UserContext,), Error = Rejection> + Clone {
    warp::header::headers_cloned()
    .map(move |headers: HeaderMap| (headers.clone(), jwt_secret.clone()))
        .and_then(|(headers, jwt_secret): (HeaderMap, String)| async move {
            match token_from_header(&headers) {
                Ok(jwt) => {
                    let decoded = decode::<Claims>(
                        &jwt,
                        &DecodingKey::from_secret(jwt_secret.as_bytes()),
                        &Validation::new(Algorithm::HS256),
                    );

                    match decoded {
                        Ok(data) => Ok(UserContext {
                            tenant_id: data.claims.tenant_id,
                            user_id: data.claims.user_id,
                        }),
                        Err(_) => Err(reject::custom(Error::InvalidToken)),
                    }
                }
                Err(_) => Err(reject::custom(Error::InvalidToken)),
            }
        })
}
```

Lots of going on here. Let's go over it step by step.

First we declare Claims struct that represents claims we defined in our JWT token using `genjwt` command line utility we wrote earlier. Notice, that this is a simple example JWT so we are not using all of the standard claims. For example, `user_id` would be `sub` in standard JWT but for now I wanted to be more expressive.

Anyway, we will also be using the same crate `jsonwebtoken` as we did in utility and we need Claims struct for the `decode` function to deserialize the token.

```rust
        match token_from_header(&headers) {
                Ok(jwt) => {
                    let decoded = decode::<Claims>(
                        &jwt,
                        &DecodingKey::from_secret(jwt_secret.as_bytes()),
                        &Validation::new(Algorithm::HS256),
                    );

                    match decoded {
                        Ok(data) => Ok(UserContext {
                            tenant_id: data.claims.tenant_id,
                            user_id: data.claims.user_id,
                        }),
                        Err(_) => Err(reject::custom(Error::InvalidToken)),
                    }
                }
                Err(_) => Err(reject::custom(Error::InvalidToken)),
```

`decode` function will also validate if JWT is valid. This includes validation of `exp` claim that determines if token is expired. If token is invalid or expired we will return `InvalidToken` error. We will also return `InvalidToken` error if we cannot deserialize the token at all.

BTW, I like to call these filter with_XXX because it makes it clear that it is a filter and it is a filter that will return XXX. So let's quickly rename `store_filter` to `with_store` in `main.rs`:

```rust
let with_store = warp::any().map(move || store_for_routes.clone());
```

We are now ready to define the route for `todo_add` handler in main.rs:

```rust
    let add_todo_route = warp::post()
        .and(warp::path("todos"))
        .and(warp::path::end())
        .and(with_jwt(jwt_secret.clone()))
        .and(with_store.clone())
        .and(warp::body::json())
        .and_then(add_todo);
```

Now, let's finish implementing the remaining APIs. Create new file `get_todo.rs` in `routes` folder:

```rust
use crate::storage::store::{TodoStore, UserContext};
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_todo(
    id: Uuid,
    user: UserContext,
    store: Arc<dyn TodoStore>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let todo = store.get_todo(&user, id.to_string()).await?;
    Ok(warp::reply::json(&todo))
}
```

and `get_todos.rs`:

```rust
use crate::storage::store::{TodoStore, UserContext};
use std::sync::Arc;

pub async fn get_todos(
    user: UserContext,
    store: Arc<dyn TodoStore>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let todos = store.get_todos(&user).await?;
    Ok(warp::reply::json(&todos))
}
```

and `update_todo.rs`:

```rust
use crate::storage::store::{TodoStore, UserContext};
use crate::model::todo::UpdateTodo;
use std::sync::Arc;
use uuid::Uuid;

pub async fn update_todo(
    id: Uuid,
    update_todo: UpdateTodo,
    user: UserContext,
    store: Arc<dyn TodoStore>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let todo = store.update_todo(&user, id.to_string(), update_todo).await?;
    Ok(warp::reply::json(&todo))
}
```

Notice that we are using `UpdateTodo` struct for `update_todo` handler. Let's add it to `model/todo.rs`:

```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateTodo {
    pub task: Option<String>,
    pub completed: Option<bool>,
}
```

In our implementation of `update_todo` in `Memstore.rs` we are only expecting `completed` field but I think it would nice if we could also rename the task. We also may want to update both fields or one or the other to do so we use `Option` type.

Let's update `store.rs` to reflect this change:

```rust
async fn update_todo(
        &self,
        ctx: &UserContext,
        id: String,
        update_todo: UpdateTodo,
    ) -> Result<Option<Todo>, Error>;
```

and update implementation in `memstore.rs`:

```rust
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
```

Finally add `delete_todo.rs`:

```rust
use crate::storage::store::{TodoStore, UserContext};
use std::sync::Arc;
use uuid::Uuid;

pub async fn delete_todo(
    id: Uuid,
    user: UserContext,
    store: Arc<dyn TodoStore>,
) -> Result<impl warp::Reply, warp::Rejection> {
    store.delete_todo(&user, id.to_string()).await?;
    Ok(warp::http::StatusCode::NO_CONTENT)
}
```

We are now ready to define the routes for these handlers in `main.rs`:

```rust
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["User-Agent", "Content-Type", "Authorization"])
        .allow_methods(&[Method::GET, Method::POST, Method::DELETE, Method::PATCH]);

    let get_todo_route = warp::get()
        .and(warp::path!("todos" / Uuid))
        .and(warp::path::end())
        .and(with_jwt(jwt_secret.clone()))
        .and(with_store.clone())
        .and_then(get_todo);

    let get_todos_route = warp::get()
        .and(warp::path("todos"))
        .and(warp::path::end())
        .and(with_jwt(jwt_secret.clone()))
        .and(with_store.clone())
        .and_then(get_todos);

    let add_todo_route = warp::post()
        .and(warp::path("todos"))
        .and(warp::path::end())
        .and(with_jwt(jwt_secret.clone()))
        .and(with_store.clone())
        .and(warp::body::json())
        .and_then(add_todo);

    let update_todo_route = warp::patch()
        .and(warp::path!("todos" / Uuid))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_jwt(jwt_secret.clone()))
        .and(with_store.clone())
        .and_then(update_todo);

    let delete_todo_route = warp::delete()
        .and(warp::path!("todos" / Uuid))
        .and(warp::path::end())
        .and(with_jwt(jwt_secret.clone()))
        .and(with_store.clone())
        .and_then(delete_todo);

    let routes = get_todo_route
        .or(get_todos_route)
        .or(add_todo_route)
        .or(update_todo_route)
        .or(delete_todo_route)
        .with(cors)
        .recover(return_error);
```
