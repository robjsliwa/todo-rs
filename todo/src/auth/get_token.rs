use super::TokenResponse;
use crate::commands::CommandContext;
use base64::Engine;
use cred_store::CredStore;
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Claims {
    exp: i64,
}

fn decode_claims_without_verification(token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = token.split('.').collect();

    if parts.len() != 3 {
        return Err("Token format is incorrect".into());
    }

    let payload = parts[1];
    let decoded_payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(payload)?;
    let claims: Claims = serde_json::from_slice(&decoded_payload)?;

    Ok(claims)
}

fn is_token_expired(token: &str) -> bool {
    let claims = match decode_claims_without_verification(token) {
        Ok(claims) => claims,
        Err(_) => return true,
    };

    let now = chrono::Utc::now().timestamp();

    claims.exp < now
}

pub fn refresh_access_token(
    domain: &str,
    client_id: &str,
    refresh_token: &str,
) -> Result<TokenResponse, Box<dyn std::error::Error>> {
    let client = Client::new();
    let token_endpoint = format!("{}/oauth/token", domain);

    let resp = client
        .post(token_endpoint)
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", refresh_token),
        ])
        .send();

    match resp {
        Ok(response) => {
            let token_response: TokenResponse = response.json()?;
            Ok(token_response)
        }
        Err(e) => Err(Box::new(e)),
    }
}

pub fn get_token(
    context: &mut CommandContext,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut credentials = context.cred_store.load()?;
    let access_token = credentials.get("access_token").cloned();
    let refresh_token = credentials.get("refresh_token").cloned();

    match (access_token, refresh_token) {
        (Some(at), Some(rt)) => {
            if is_token_expired(&at) {
                let token_response =
                    refresh_access_token(&context.config.domain, &context.config.client_id, &rt)?;
                let new_access_token = token_response.access_token.unwrap();
                let new_refresh_token = token_response.refresh_token.unwrap();

                credentials
                    .add("access_token".to_string(), new_access_token.clone())
                    .add("refresh_token".to_string(), new_refresh_token);

                credentials.save()?;

                Ok(Some(new_access_token))
            } else {
                Ok(Some(at))
            }
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_claims_without_verification() {
        let test_token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ0ZW5hbnRfaWQiOiIxIiwidXNlcl9pZCI6IjEiLCJleHAiOjE2OTcxMTg2Nzh9.CYF2GjJ5T1xJSUM5T1gl9iFftufT8xe8cclGoU8kw_I";
        let claims = decode_claims_without_verification(test_token).unwrap();
        assert_eq!(claims.exp, 1697118678);
    }
}
