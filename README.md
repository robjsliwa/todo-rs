# todo-rs

This repo is a companion for my articles: Building a Multi-Tenant To do Server in Rust

- [Part 1](https://medium.com/@robjsliwa_71070/building-a-multi-tenant-to-do-server-in-rust-part-1-4b90c0604224)
- [Part 2](https://medium.com/@robjsliwa_71070/building-a-multi-tenant-todo-server-in-rust-part-2-58e2ec137c87)
- [Part 3](https://medium.com/@robjsliwa_71070/building-a-multi-tenant-to-do-server-in-rust-part-3-6a78c47f800d)
- [Part 4](https://medium.com/@robjsliwa_71070/crafting-cli-with-oauth-2-0-authentication-multi-tenant-todo-server-in-rust-series-eaa0af452a56)
- [Part 5](https://medium.com/dev-genius/simplifying-container-based-development-of-rust-microservices-with-tilt-eb2fd0a48e3e)
- Part 6 (coming soon)

# How to run

1. Clone the repo
2. Install [Docker](https://docs.docker.com/install/) and [Docker Compose](https://docs.docker.com/compose/install/)
3. Install [Tilt](https://docs.tilt.dev/install.html)
4. Create a `.env` file in the root of the repo with the following contents:

```
JWT_SECRET=your_secret
TODO_PORT=3030
MONGO_URI=mongodb://mongodb:27017
AUTH0_DOMAIN=https://<auth0 domain>
AUTH0_AUDIENCE=https://<auth0 audience>

5. Run `tilt up` in the root of the repo.
