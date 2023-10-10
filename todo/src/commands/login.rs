use cred_store::CredStore;

use super::CommandContext;
use crate::auth;

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

pub fn login(context: &mut CommandContext) {
    match auth::login(context.config) {
        Ok(resp) => {
            let access_token = resp.access_token.clone().unwrap();
            let refresh_token = resp.refresh_token.clone().unwrap();
            println!();
            println!("Access Token: {}", access_token);
            if save_tokens(&access_token, &refresh_token, context).is_err() {
                println!("Couldn't configure credentials.");
            }
        }
        Err(e) => println!("Error logging in: {}", e),
    }
}
