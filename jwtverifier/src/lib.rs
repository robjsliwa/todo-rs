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
    let token =
        jsonwebtoken::decode::<Claims>(jwt, &DecodingKey::from_jwk(&jwk)?, &validation)?;
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
        let url = "https://dev-ogo6abmw5x0hsuer.us.auth0.com/.well-known/jwks.json";
        let resp = fetch_jwt(url).await.unwrap();
        println!("{:#?}", resp);
        assert_eq!(resp.keys.len(), 2);
    }

    #[tokio::test]
    async fn test_verify_jwt() {
        let jwt = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjF6dTE3U0VDdmhfWmNnNHM5UVBxWCJ9.eyJpc3MiOiJodHRwczovL2Rldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbS8iLCJzdWIiOiJhdXRoMHw2NTEyY2U1MzUxODYwNDlmYjJhOTAxODEiLCJhdWQiOlsiaHR0cHM6Ly90b2Rvcy5leGFtcGxlLmNvbS8iLCJodHRwczovL2Rldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbS91c2VyaW5mbyJdLCJpYXQiOjE2OTY1NDg1MjIsImV4cCI6MTY5NjYzNDkyMiwiYXpwIjoiRlFRTjJRVmRobldQb1M3eFZqOGp2SnZTWU1oSDNYVVQiLCJzY29wZSI6Im9wZW5pZCBwcm9maWxlIGVtYWlsIG9mZmxpbmVfYWNjZXNzIn0.XIYlnnVPEcJLu3EQlMJFMhH2h-cWtsSbZWBSy7i2X5H3far2w2PTVV2NuC1ajxrdrCoiPwizBmHK1qnNP8vKLlJ5_4cRZX5Weptpinz61TsBIHbDd4W9iojrGMQTFWgpbz50dvhXB6C4mpjB043zEPdm4OVJ9OHzYegYJ-o2Qa56Q92eDHfGp0iINJ5ZzCx9ojfqnOPFyfL5GMhh1X4ca5Nnh7p8knUCx-Th4W7BJdVR52_qp7phX0M8s1847nj7iIyzrnRS7ZNfcG-Lh3N2_6uvT0-IqaqbTb-vNF8fqlrppg0cVA6ha2I42tJjPi49Mm06gvY6JKcO4alr3IfgWw";
        let jwks = fetch_jwt("https://dev-ogo6abmw5x0hsuer.us.auth0.com/.well-known/jwks.json")
            .await
            .unwrap();
        let aud = "https://todos.example.com/";
        let resp = verify_jwt::<Claims>(jwt, &jwks, aud).await;
        println!("{:#?}", resp);
        assert!(resp.is_ok());
    }
}
