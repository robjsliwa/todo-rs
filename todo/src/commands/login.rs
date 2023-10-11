use cred_store::CredStore;

use super::CommandContext;
use crate::auth;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserInfo {
    external_id: String,
    email: String,
    name: String,
    id: String,
    tenant_id: String,
}

fn save_tokens(
    access_token: &str,
    refresh_token: &str,
    context: &mut CommandContext,
) -> Result<(), std::io::Error> {
    context
        .cred_store
        .clear()
        .add("access_token".to_string(), access_token.to_string())
        .add("refresh_token".to_string(), refresh_token.to_string())
        .save()
}

fn get_userinfo(url: &str, access_token: &str) -> Result<UserInfo, Box<dyn std::error::Error>> {
    let client = Client::new();
    let resp = client
        .get(format!("{}/userinfo", url))
        .bearer_auth(access_token)
        .send()?;

    let userinfo: UserInfo = match resp.json::<UserInfo>() {
        Ok(resp) => resp,
        Err(e) => return Err(e.into()),
    };

    Ok(userinfo)
}

pub fn login(context: &mut CommandContext) {
    match auth::login(context.config) {
        Ok(resp) => {
            let access_token = resp.access_token.clone().unwrap();
            let refresh_token = resp.refresh_token.clone().unwrap();
            println!();
            println!("Access Token: {}", access_token);
            if save_tokens(&access_token, &refresh_token, context).is_err() {
                eprintln!("Couldn't configure credentials.");
                std::process::exit(1);
            }
            match get_userinfo(&context.config.todo_url, &access_token) {
                Ok(userinfo) => {
                    println!("User Info: {:?}", userinfo);
                }
                Err(e) => {
                    eprintln!("Couldn't get user info: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => println!("Error logging in: {}", e),
    }
}
