# Building a Multi-Tenant Todo Server in Rust: Part 2

## Introduction

In Part 1 of this series, we began our journey into building a multi-tenant Todo server using Rust. We set up the basic project structure, implemented Warp routes, and even provided an in-memory storage solution. In this second installment, we're going to extend our application by:

- Reorganizing our codebase into a modular folder structure
- Defining the Todo model with multi-tenancy in mind
- Implementing RESTful APIs for our Todo server
- Protect access to our APIs with authentication
- Introducing a hexagonal architecture for our storage layer

So, let's get started!