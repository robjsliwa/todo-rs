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

We will need to consider multi-tenancy at two levels.  One at the API server level and the second one at the database level.  There are couple archtictural patterns that we can use to achieve multi-tenancy.

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

At the API server level, we will ensure that each user accesses resources that belong to them for the tenant that they are logged in by using JWT tokens with custom claims that store user id and tenant id.  Server will be able to use these two values to ensure that the user has access only to the resources they are authorized for.

At the database level we will use single database, single schema approach.  What that means from perspective of schema design is that we will have to add tenant id and user id to each table.  This will allow us to ensure that each user can only access resources that belong to them.

## Authentication

Let's begin by adding authentication to our Todo server. We'll be using [JSON Web Tokens](https://jwt.io/) (JWT) to authenticate users. JWTs are a popular choice for authentication because they are stateless, meaning that the server doesn't need to store any session information. This makes JWTs a great choice for our multi-tenant Todo server because we don't need to worry about storing session information for each tenant.

To keep simple for now we will create JWT tokens ourselves for now.  In future articles we will extend the project to use a third party authentication provider like Auth0.

### Creating JWT Tokens

This is an opportunity to create simple command line tool that will accept user id and tenant id and generate JWT token for us.  We will use [jsonwebtoken](https://crates.io/crates/jsonwebtoken) crate to generate JWT tokens.

In addition, to the above parameters we need to provide token duration.  Duration will be in seconds.  We will use 1 hour as defualt duration.

Final parameter is the secret that will be used to sign the token.  When the server receives the token it will use the same secret to verify the token.

To implement command line utility let's create new binary.  Execute the following command from project's root folder:

```bash
cargo new --bin genjwt
```

This will create new folder called `genjwt` with `main.rs` file inside.  Let's update `main.rs` file to accept user id, tenant id, duration and secret as command line arguments.  For handling command line arguments we will use [clap](https://crates.io/crates/clap) crate.

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

