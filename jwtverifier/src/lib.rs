use jsonwebtoken::{jwk::JwkSet, DecodingKey, TokenData};
use serde::de::DeserializeOwned;

pub async fn fetch_jwt(url: &str) -> Result<JwkSet, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?.json::<JwkSet>().await?;
    Ok(resp)
}

pub async fn verify_jwt<Claims: DeserializeOwned>(
    jwt: &str,
    jwks: &JwkSet,
    aud: &str,
) -> Result<TokenData<Claims>, Box<dyn std::error::Error>> {
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_audience(&[aud]);
    let header = jsonwebtoken::decode_header(jwt)?;
    let kid = match header.kid {
        Some(kid) => kid,
        None => {
            return Err("kid not found in jwt header".into());
        }
    };
    // find jwk with kid
    let jwk = match jwks.find(&kid) {
        Some(jwk) => jwk,
        None => {
            return Err("jwk not found".into());
        }
    };
    let token = jsonwebtoken::decode::<Claims>(jwt, &DecodingKey::from_jwk(&jwk)?, &validation)?;
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Claims {
        iss: String,
        sub: String,
        aud: Vec<String>,
        iat: usize,
        exp: usize,
        azp: String,
        scope: String,
    }

    #[tokio::test]
    async fn test_fetch_jwt() {
        let url = "";
        let resp = fetch_jwt(url).await.unwrap();
        println!("{:#?}", resp);
        assert_eq!(resp.keys.len(), 2);
    }

    #[tokio::test]
    async fn test_verify_jwt() {
        let jwt = "";
        let jwks = fetch_jwt("").await.unwrap();
        let aud = "https://todos.example.com/";
        let resp = verify_jwt::<Claims>(jwt, &jwks, aud).await;
        println!("{:#?}", resp);
        assert!(resp.is_ok());
    }
}
