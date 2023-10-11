use cred_store::CredStore;

use super::CommandContext;
use crate::auth;
use reqwest::blocking::Client;

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

fn get_userinfo(url: &str, access_token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let resp = client
        .get(format!("{}/userinfo", url))
        .bearer_auth(access_token)
        .send()?;

    println!("Response: {:#?}", resp);

    let response = match resp.json::<serde_json::Value>() {
        Ok(resp) => resp,
        Err(e) => return Err(e.into()),
    };

    println!("User Info: {:#?}", response);

    Ok(())
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
            }
            if let Err(e) = get_userinfo(&context.config.todo_url, &access_token) {
                eprintln!("Couldn't get user info: {}", e);
            }
        }
        Err(e) => println!("Error logging in: {}", e),
    }
}
