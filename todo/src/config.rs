use dotenv::dotenv;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub domain: String,
    pub client_id: String,
    pub audience: String,
    pub todo_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv().ok();
        let domain = env::var("DOMAIN")?;
        let client_id = env::var("CLIENT_ID")?;
        let audience = env::var("AUDIENCE")?;
        let todo_url = env::var("TODO_URL")?;
        println!("domain: {}", domain);

        Ok(Self {
            domain,
            client_id,
            audience,
            todo_url,
        })
    }
}
