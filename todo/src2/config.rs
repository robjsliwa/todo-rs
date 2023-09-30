use dotenv::dotenv;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub domain: String,
    pub client_id: String,
    pub audience: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv().ok();
        let domain = env::var("DOMAIN")?;
        let client_id = env::var("CLIENT_ID")?;
        let audience = env::var("AUDIENCE")?;
        println!("domain: {}", domain);

        Ok(Self {
            domain,
            client_id,
            audience,
        })
    }
}
