use jsonwebtoken::{jwk::JwkSet, DecodingKey, TokenData};
use serde::de::DeserializeOwned;

const JWKS_URI: &str = ".well-known/jwks.json";

pub struct JwtVerifier {
    domain: String,
    jwks_cache: Option<JwkSet>,
    use_cache: bool,
}

impl JwtVerifier {
    pub fn new(domain: &str) -> Self {
        Self {
            domain: domain.to_string(),
            jwks_cache: None,
            use_cache: false,
        }
    }

    pub fn use_cache(mut self, value: bool) -> Self {
        self.use_cache = value;
        self
    }

    pub fn build(self) -> JwtVerifier {
        JwtVerifier {
            domain: self.domain,
            jwks_cache: self.jwks_cache,
            use_cache: self.use_cache,
        }
    }

    pub async fn verify<Claims: DeserializeOwned>(
        mut self,
        jwt: &str,
        aud: &str,
    ) -> Result<TokenData<Claims>, Box<dyn std::error::Error>> {
        let jwks = match self.use_cache {
            true => match &mut self.jwks_cache {
                Some(jwks) => jwks.clone(),
                None => {
                    let jwks = fetch_jwt(&format!("{}/{}", self.domain, JWKS_URI)).await?;
                    self.jwks_cache = Some(jwks.clone());
                    jwks
                }
            },
            false => fetch_jwt(&format!("{}/{}", self.domain, JWKS_URI)).await?,
        };
        verify_jwt(jwt, &jwks, aud).await
    }
}

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
    let token = jsonwebtoken::decode::<Claims>(jwt, &DecodingKey::from_jwk(jwk)?, &validation)?;
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct Claims {
        pub iss: String,
        pub sub: String,
        pub aud: Vec<String>,
        pub iat: usize,
        pub exp: usize,
        pub azp: String,
        pub scope: String,
    }

    #[tokio::test]
    async fn test_fetch_jwt() {
        // Set up the mock
        let _m = mock("GET", "/.well-known/jwks.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"keys":[{"kty":"RSA","use":"sig","n":"7Z89Y4HjYOWQlePNfPFAiL24SG9GdPtiPF6SjQVe5X26KNQrpT0vBGGsfixbQ5NoBpXviFk8qHXi1cdyBwqr8eve8hEo9Kw91_NTco1BM2hIs3kSttfvRKg9ySfV0T4c0kuDdVVlZSNh2l1jOHqeM5oYhL-Ujq9jIG-JAy63WZx_lmsQN_5adHgNBT54YgEW9oNBl4MTSeFbA1ffDrXbW0OtqktiveCHQGI17_eE-RytNZ5PwCL2D793lNDf3sRNY4r4_VVDrF84En3Jr_rY6ogzxN3LSw43ewFOP0igRps4ZmVrzHvqrjbHn8in0sO6mICwsaBthn4oF92AtKDoKw","e":"AQAB","kid":"1zu17SECvh_Zcg4s9QPqX","x5t":"Vx_J2QjyEk-0NXQvF-thh29n6Q8","x5c":["MIIDHTCCAgWgAwIBAgIJOV8w2KgE5VN5MA0GCSqGSIb3DQEBCwUAMCwxKjAoBgNVBAMTIWRldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbTAeFw0yMjExMjMxMzI1NThaFw0zNjA4MDExMzI1NThaMCwxKjAoBgNVBAMTIWRldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbTCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAO2fPWOB42DlkJXjzXzxQIi9uEhvRnT7Yjxeko0FXuV9uijUK6U9LwRhrH4sW0OTaAaV74hZPKh14tXHcgcKq/Hr3vIRKPSsPdfzU3KNQTNoSLN5ErbX70SoPckn1dE+HNJLg3VVZWUjYdpdYzh6njOaGIS/lI6vYyBviQMut1mcf5ZrEDf+WnR4DQU+eGIBFvaDQZeDE0nhWwNX3w6121tDrapLYr3gh0BiNe/3hPkcrTWeT8Ai9g+/d5TQ397ETWOK+P1VQ6xfOBJ9ya/62OqIM8Tdy0sON3sBTj9IoEabOGZla8x76q42x5/Ip9LDupiAsLGgbYZ+KBfdgLSg6CsCAwEAAaNCMEAwDwYDVR0TAQH/BAUwAwEB/zAdBgNVHQ4EFgQUbMuvaPAXW0x0UIs2PQRrjN4mvJIwDgYDVR0PAQH/BAQDAgKEMA0GCSqGSIb3DQEBCwUAA4IBAQAHUIHuNR309kVV5vDCBIOr/NqmACT1ADh83cGMjc2KfYmdWt0iaR2QQdToXSZx8y6QKeGaZ77696na0OdYDkf/ngYX7YovhgsDgy65h+c2o+myIgeViWIZvqCt7+v+7kCw1DNkEhwYQx7/4DWf91uOqQmDGkrEFbk2h/2e0TmhYFgg9isBQ0+lWdL2kutdaoC+a+I3krIdLKqHgqQbs+d57y4/h6rHmZMv55WGXvKN21wu6JcMmzFkB1GNrJ/Ce7nIWRa0Kz5RVn4Yuq6BK18yTFI3w227i1Jz440Ce4eumQ0zsaEl+ZYNcJ9MU5sqUI3gji582nIkWHf42A692ZTC"],"alg":"RS256"},{"kty":"RSA","use":"sig","n":"xDG7pvlsuNrJ4AkOs2MZY9zpw4Qlqqbg5pXUhPbu33ahl27WU8M1zzkbne2i6_aHV71NcHp_C_OYzvo9-zw-AWHKj6UTp6JXca5MJJcE3djiHVbyCz0Du2MWQX_YDZb_2LncjbmnSbmIgN83k5vntBg-k4bJHR7RBkm5GDR7rSEUxGfJ7lOFgKY5HI4xIluk6u6YZ91GQK1BFi3kk_tBysyHZQMHp3A_vf584uYV42Kz6pJb-ZAZ94ZdIvxOUENSgEGwaA3qS1F8yByNg6n9axlTaN37XU8NBu4nld4w5XdTrvRyIxVrz8MfXRl6ILup1pNMeupx4SKlH_6i64juMw","e":"AQAB","kid":"v8NYxpog922LekQ_geMou","x5t":"Fy6Iq7McnGKDrlvwm2xpan4qOAo","x5c":["MIIDHTCCAgWgAwIBAgIJOHyUS8nhvDq/MA0GCSqGSIb3DQEBCwUAMCwxKjAoBgNVBAMTIWRldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbTAeFw0yMjExMjMxMzI1NThaFw0zNjA4MDExMzI1NThaMCwxKjAoBgNVBAMTIWRldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbTCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAMQxu6b5bLjayeAJDrNjGWPc6cOEJaqm4OaV1IT27t92oZdu1lPDNc85G53touv2h1e9TXB6fwvzmM76Pfs8PgFhyo+lE6eiV3GuTCSXBN3Y4h1W8gs9A7tjFkF/2A2W/9i53I25p0m5iIDfN5Ob57QYPpOGyR0e0QZJuRg0e60hFMRnye5ThYCmORyOMSJbpOrumGfdRkCtQRYt5JP7QcrMh2UDB6dwP73+fOLmFeNis+qSW/mQGfeGXSL8TlBDUoBBsGgN6ktRfMgcjYOp/WsZU2jd+11PDQbuJ5XeMOV3U670ciMVa8/DH10ZeiC7qdaTTHrqceEipR/+ouuI7jMCAwEAAaNCMEAwDwYDVR0TAQH/BAUwAwEB/zAdBgNVHQ4EFgQU3C3hbxhquy/RGdSdUy0pe/pRSXAwDgYDVR0PAQH/BAQDAgKEMA0GCSqGSIb3DQEBCwUAA4IBAQBwpMJXoTmkqkLogUgjXKP2V3bj8A9BUlZ3HWazblEIhjqXE84BwFdYLOozTsVPaUEjeGilRq28sBt/qkPCkZRi4JSd4Kiuri69NfYSPgW1rZrVBpkHylPwp0XNkBnu5xczU5184/3VNgv2czOsmWj4EP0OgBGHwTXB9/POQPP11rUzz0N/sv7uv4xrnAov5W/33alVm9GKga958/S75fUantzq6vBBLhmbWuwnqCE6o6a4axpU7HA67B6+QSoxZcHauq2rdbJgtksEfGGitBY5lle25SOKAZ+tHj0ZJnm5dx6etOhhk1k96sr8fP7qpOkgEXOJLZ0fvr6Pj+U12w6K"],"alg":"RS256"}]}"#)
            .create();

        let url = &format!("{}/.well-known/jwks.json", "http://localhost:1234");
        let resp = fetch_jwt(url).await.unwrap();
        println!("{:#?}", resp);
        assert_eq!(resp.keys.len(), 2);
    }

    #[tokio::test]
    async fn test_verify_jwt() {
        // Set up the mock
        let _m = mock("GET", "/.well-known/jwks.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"keys":[{"kty":"RSA","use":"sig","n":"7Z89Y4HjYOWQlePNfPFAiL24SG9GdPtiPF6SjQVe5X26KNQrpT0vBGGsfixbQ5NoBpXviFk8qHXi1cdyBwqr8eve8hEo9Kw91_NTco1BM2hIs3kSttfvRKg9ySfV0T4c0kuDdVVlZSNh2l1jOHqeM5oYhL-Ujq9jIG-JAy63WZx_lmsQN_5adHgNBT54YgEW9oNBl4MTSeFbA1ffDrXbW0OtqktiveCHQGI17_eE-RytNZ5PwCL2D793lNDf3sRNY4r4_VVDrF84En3Jr_rY6ogzxN3LSw43ewFOP0igRps4ZmVrzHvqrjbHn8in0sO6mICwsaBthn4oF92AtKDoKw","e":"AQAB","kid":"1zu17SECvh_Zcg4s9QPqX","x5t":"Vx_J2QjyEk-0NXQvF-thh29n6Q8","x5c":["MIIDHTCCAgWgAwIBAgIJOV8w2KgE5VN5MA0GCSqGSIb3DQEBCwUAMCwxKjAoBgNVBAMTIWRldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbTAeFw0yMjExMjMxMzI1NThaFw0zNjA4MDExMzI1NThaMCwxKjAoBgNVBAMTIWRldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbTCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAO2fPWOB42DlkJXjzXzxQIi9uEhvRnT7Yjxeko0FXuV9uijUK6U9LwRhrH4sW0OTaAaV74hZPKh14tXHcgcKq/Hr3vIRKPSsPdfzU3KNQTNoSLN5ErbX70SoPckn1dE+HNJLg3VVZWUjYdpdYzh6njOaGIS/lI6vYyBviQMut1mcf5ZrEDf+WnR4DQU+eGIBFvaDQZeDE0nhWwNX3w6121tDrapLYr3gh0BiNe/3hPkcrTWeT8Ai9g+/d5TQ397ETWOK+P1VQ6xfOBJ9ya/62OqIM8Tdy0sON3sBTj9IoEabOGZla8x76q42x5/Ip9LDupiAsLGgbYZ+KBfdgLSg6CsCAwEAAaNCMEAwDwYDVR0TAQH/BAUwAwEB/zAdBgNVHQ4EFgQUbMuvaPAXW0x0UIs2PQRrjN4mvJIwDgYDVR0PAQH/BAQDAgKEMA0GCSqGSIb3DQEBCwUAA4IBAQAHUIHuNR309kVV5vDCBIOr/NqmACT1ADh83cGMjc2KfYmdWt0iaR2QQdToXSZx8y6QKeGaZ77696na0OdYDkf/ngYX7YovhgsDgy65h+c2o+myIgeViWIZvqCt7+v+7kCw1DNkEhwYQx7/4DWf91uOqQmDGkrEFbk2h/2e0TmhYFgg9isBQ0+lWdL2kutdaoC+a+I3krIdLKqHgqQbs+d57y4/h6rHmZMv55WGXvKN21wu6JcMmzFkB1GNrJ/Ce7nIWRa0Kz5RVn4Yuq6BK18yTFI3w227i1Jz440Ce4eumQ0zsaEl+ZYNcJ9MU5sqUI3gji582nIkWHf42A692ZTC"],"alg":"RS256"},{"kty":"RSA","use":"sig","n":"xDG7pvlsuNrJ4AkOs2MZY9zpw4Qlqqbg5pXUhPbu33ahl27WU8M1zzkbne2i6_aHV71NcHp_C_OYzvo9-zw-AWHKj6UTp6JXca5MJJcE3djiHVbyCz0Du2MWQX_YDZb_2LncjbmnSbmIgN83k5vntBg-k4bJHR7RBkm5GDR7rSEUxGfJ7lOFgKY5HI4xIluk6u6YZ91GQK1BFi3kk_tBysyHZQMHp3A_vf584uYV42Kz6pJb-ZAZ94ZdIvxOUENSgEGwaA3qS1F8yByNg6n9axlTaN37XU8NBu4nld4w5XdTrvRyIxVrz8MfXRl6ILup1pNMeupx4SKlH_6i64juMw","e":"AQAB","kid":"v8NYxpog922LekQ_geMou","x5t":"Fy6Iq7McnGKDrlvwm2xpan4qOAo","x5c":["MIIDHTCCAgWgAwIBAgIJOHyUS8nhvDq/MA0GCSqGSIb3DQEBCwUAMCwxKjAoBgNVBAMTIWRldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbTAeFw0yMjExMjMxMzI1NThaFw0zNjA4MDExMzI1NThaMCwxKjAoBgNVBAMTIWRldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbTCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAMQxu6b5bLjayeAJDrNjGWPc6cOEJaqm4OaV1IT27t92oZdu1lPDNc85G53touv2h1e9TXB6fwvzmM76Pfs8PgFhyo+lE6eiV3GuTCSXBN3Y4h1W8gs9A7tjFkF/2A2W/9i53I25p0m5iIDfN5Ob57QYPpOGyR0e0QZJuRg0e60hFMRnye5ThYCmORyOMSJbpOrumGfdRkCtQRYt5JP7QcrMh2UDB6dwP73+fOLmFeNis+qSW/mQGfeGXSL8TlBDUoBBsGgN6ktRfMgcjYOp/WsZU2jd+11PDQbuJ5XeMOV3U670ciMVa8/DH10ZeiC7qdaTTHrqceEipR/+ouuI7jMCAwEAAaNCMEAwDwYDVR0TAQH/BAUwAwEB/zAdBgNVHQ4EFgQU3C3hbxhquy/RGdSdUy0pe/pRSXAwDgYDVR0PAQH/BAQDAgKEMA0GCSqGSIb3DQEBCwUAA4IBAQBwpMJXoTmkqkLogUgjXKP2V3bj8A9BUlZ3HWazblEIhjqXE84BwFdYLOozTsVPaUEjeGilRq28sBt/qkPCkZRi4JSd4Kiuri69NfYSPgW1rZrVBpkHylPwp0XNkBnu5xczU5184/3VNgv2czOsmWj4EP0OgBGHwTXB9/POQPP11rUzz0N/sv7uv4xrnAov5W/33alVm9GKga958/S75fUantzq6vBBLhmbWuwnqCE6o6a4axpU7HA67B6+QSoxZcHauq2rdbJgtksEfGGitBY5lle25SOKAZ+tHj0ZJnm5dx6etOhhk1k96sr8fP7qpOkgEXOJLZ0fvr6Pj+U12w6K"],"alg":"RS256"}]}"#)
            .create();

        let jwt = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjF6dTE3U0VDdmhfWmNnNHM5UVBxWCJ9.eyJpc3MiOiJodHRwczovL2Rldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbS8iLCJzdWIiOiJhdXRoMHw2NTEyY2U1MzUxODYwNDlmYjJhOTAxODEiLCJhdWQiOlsiaHR0cHM6Ly90b2Rvcy5leGFtcGxlLmNvbS8iLCJodHRwczovL2Rldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbS91c2VyaW5mbyJdLCJpYXQiOjE2OTY2Mzk5MjUsImV4cCI6MTY5NjcyNjMyNSwiYXpwIjoiRlFRTjJRVmRobldQb1M3eFZqOGp2SnZTWU1oSDNYVVQiLCJzY29wZSI6Im9wZW5pZCBwcm9maWxlIGVtYWlsIG9mZmxpbmVfYWNjZXNzIn0.Q65UjlmbHHcDL7WIHTQ30Zy6PFi46bfxaJBu8pxcRtUiQzWugj6kkwt9FsCyStCJhahcWIZDfrtHBaweH3ynkS4n05HXYBtuUAK-hbWgR-NcXY31z9HdiSjY67gpYUoLvbuwytSlmh7rryN80jUR9HpivKtfN9i-6A45gf1R14TzkPKxmvDLRIGHiSnlqM7WFitEUfRCkaRuV4SEVyGRpX4VHwVBq7e5m2SoEPuNOnRenl56VmROcJhXBwNvdBzqrYkWDDx_pvZbY0iPeFiUL3pVzdQh_PCHtWq25nNKGFGm3hxMPloNXkHsqncDgMl2y08fMGf0e07c3ALv-YmVKw";
        let jwks = fetch_jwt(&format!(
            "{}/.well-known/jwks.json",
            "http://localhost:1234"
        ))
        .await
        .unwrap();
        let aud = "https://todos.example.com/";
        let resp = verify_jwt::<Claims>(jwt, &jwks, aud).await;
        println!("{:#?}", resp);
        assert!(resp.is_err());
        assert_eq!(resp.unwrap_err().to_string(), "ExpiredSignature");
    }

    #[tokio::test]
    async fn test_jwt_verifier() {
        // Set up the mock
        let _m = mock("GET", "/.well-known/jwks.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"keys":[{"kty":"RSA","use":"sig","n":"7Z89Y4HjYOWQlePNfPFAiL24SG9GdPtiPF6SjQVe5X26KNQrpT0vBGGsfixbQ5NoBpXviFk8qHXi1cdyBwqr8eve8hEo9Kw91_NTco1BM2hIs3kSttfvRKg9ySfV0T4c0kuDdVVlZSNh2l1jOHqeM5oYhL-Ujq9jIG-JAy63WZx_lmsQN_5adHgNBT54YgEW9oNBl4MTSeFbA1ffDrXbW0OtqktiveCHQGI17_eE-RytNZ5PwCL2D793lNDf3sRNY4r4_VVDrF84En3Jr_rY6ogzxN3LSw43ewFOP0igRps4ZmVrzHvqrjbHn8in0sO6mICwsaBthn4oF92AtKDoKw","e":"AQAB","kid":"1zu17SECvh_Zcg4s9QPqX","x5t":"Vx_J2QjyEk-0NXQvF-thh29n6Q8","x5c":["MIIDHTCCAgWgAwIBAgIJOV8w2KgE5VN5MA0GCSqGSIb3DQEBCwUAMCwxKjAoBgNVBAMTIWRldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbTAeFw0yMjExMjMxMzI1NThaFw0zNjA4MDExMzI1NThaMCwxKjAoBgNVBAMTIWRldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbTCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAO2fPWOB42DlkJXjzXzxQIi9uEhvRnT7Yjxeko0FXuV9uijUK6U9LwRhrH4sW0OTaAaV74hZPKh14tXHcgcKq/Hr3vIRKPSsPdfzU3KNQTNoSLN5ErbX70SoPckn1dE+HNJLg3VVZWUjYdpdYzh6njOaGIS/lI6vYyBviQMut1mcf5ZrEDf+WnR4DQU+eGIBFvaDQZeDE0nhWwNX3w6121tDrapLYr3gh0BiNe/3hPkcrTWeT8Ai9g+/d5TQ397ETWOK+P1VQ6xfOBJ9ya/62OqIM8Tdy0sON3sBTj9IoEabOGZla8x76q42x5/Ip9LDupiAsLGgbYZ+KBfdgLSg6CsCAwEAAaNCMEAwDwYDVR0TAQH/BAUwAwEB/zAdBgNVHQ4EFgQUbMuvaPAXW0x0UIs2PQRrjN4mvJIwDgYDVR0PAQH/BAQDAgKEMA0GCSqGSIb3DQEBCwUAA4IBAQAHUIHuNR309kVV5vDCBIOr/NqmACT1ADh83cGMjc2KfYmdWt0iaR2QQdToXSZx8y6QKeGaZ77696na0OdYDkf/ngYX7YovhgsDgy65h+c2o+myIgeViWIZvqCt7+v+7kCw1DNkEhwYQx7/4DWf91uOqQmDGkrEFbk2h/2e0TmhYFgg9isBQ0+lWdL2kutdaoC+a+I3krIdLKqHgqQbs+d57y4/h6rHmZMv55WGXvKN21wu6JcMmzFkB1GNrJ/Ce7nIWRa0Kz5RVn4Yuq6BK18yTFI3w227i1Jz440Ce4eumQ0zsaEl+ZYNcJ9MU5sqUI3gji582nIkWHf42A692ZTC"],"alg":"RS256"},{"kty":"RSA","use":"sig","n":"xDG7pvlsuNrJ4AkOs2MZY9zpw4Qlqqbg5pXUhPbu33ahl27WU8M1zzkbne2i6_aHV71NcHp_C_OYzvo9-zw-AWHKj6UTp6JXca5MJJcE3djiHVbyCz0Du2MWQX_YDZb_2LncjbmnSbmIgN83k5vntBg-k4bJHR7RBkm5GDR7rSEUxGfJ7lOFgKY5HI4xIluk6u6YZ91GQK1BFi3kk_tBysyHZQMHp3A_vf584uYV42Kz6pJb-ZAZ94ZdIvxOUENSgEGwaA3qS1F8yByNg6n9axlTaN37XU8NBu4nld4w5XdTrvRyIxVrz8MfXRl6ILup1pNMeupx4SKlH_6i64juMw","e":"AQAB","kid":"v8NYxpog922LekQ_geMou","x5t":"Fy6Iq7McnGKDrlvwm2xpan4qOAo","x5c":["MIIDHTCCAgWgAwIBAgIJOHyUS8nhvDq/MA0GCSqGSIb3DQEBCwUAMCwxKjAoBgNVBAMTIWRldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbTAeFw0yMjExMjMxMzI1NThaFw0zNjA4MDExMzI1NThaMCwxKjAoBgNVBAMTIWRldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbTCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAMQxu6b5bLjayeAJDrNjGWPc6cOEJaqm4OaV1IT27t92oZdu1lPDNc85G53touv2h1e9TXB6fwvzmM76Pfs8PgFhyo+lE6eiV3GuTCSXBN3Y4h1W8gs9A7tjFkF/2A2W/9i53I25p0m5iIDfN5Ob57QYPpOGyR0e0QZJuRg0e60hFMRnye5ThYCmORyOMSJbpOrumGfdRkCtQRYt5JP7QcrMh2UDB6dwP73+fOLmFeNis+qSW/mQGfeGXSL8TlBDUoBBsGgN6ktRfMgcjYOp/WsZU2jd+11PDQbuJ5XeMOV3U670ciMVa8/DH10ZeiC7qdaTTHrqceEipR/+ouuI7jMCAwEAAaNCMEAwDwYDVR0TAQH/BAUwAwEB/zAdBgNVHQ4EFgQU3C3hbxhquy/RGdSdUy0pe/pRSXAwDgYDVR0PAQH/BAQDAgKEMA0GCSqGSIb3DQEBCwUAA4IBAQBwpMJXoTmkqkLogUgjXKP2V3bj8A9BUlZ3HWazblEIhjqXE84BwFdYLOozTsVPaUEjeGilRq28sBt/qkPCkZRi4JSd4Kiuri69NfYSPgW1rZrVBpkHylPwp0XNkBnu5xczU5184/3VNgv2czOsmWj4EP0OgBGHwTXB9/POQPP11rUzz0N/sv7uv4xrnAov5W/33alVm9GKga958/S75fUantzq6vBBLhmbWuwnqCE6o6a4axpU7HA67B6+QSoxZcHauq2rdbJgtksEfGGitBY5lle25SOKAZ+tHj0ZJnm5dx6etOhhk1k96sr8fP7qpOkgEXOJLZ0fvr6Pj+U12w6K"],"alg":"RS256"}]}"#)
            .create();

        let jwt = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjF6dTE3U0VDdmhfWmNnNHM5UVBxWCJ9.eyJpc3MiOiJodHRwczovL2Rldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbS8iLCJzdWIiOiJhdXRoMHw2NTEyY2U1MzUxODYwNDlmYjJhOTAxODEiLCJhdWQiOlsiaHR0cHM6Ly90b2Rvcy5leGFtcGxlLmNvbS8iLCJodHRwczovL2Rldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbS91c2VyaW5mbyJdLCJpYXQiOjE2OTY2Mzk5MjUsImV4cCI6MTY5NjcyNjMyNSwiYXpwIjoiRlFRTjJRVmRobldQb1M3eFZqOGp2SnZTWU1oSDNYVVQiLCJzY29wZSI6Im9wZW5pZCBwcm9maWxlIGVtYWlsIG9mZmxpbmVfYWNjZXNzIn0.Q65UjlmbHHcDL7WIHTQ30Zy6PFi46bfxaJBu8pxcRtUiQzWugj6kkwt9FsCyStCJhahcWIZDfrtHBaweH3ynkS4n05HXYBtuUAK-hbWgR-NcXY31z9HdiSjY67gpYUoLvbuwytSlmh7rryN80jUR9HpivKtfN9i-6A45gf1R14TzkPKxmvDLRIGHiSnlqM7WFitEUfRCkaRuV4SEVyGRpX4VHwVBq7e5m2SoEPuNOnRenl56VmROcJhXBwNvdBzqrYkWDDx_pvZbY0iPeFiUL3pVzdQh_PCHtWq25nNKGFGm3hxMPloNXkHsqncDgMl2y08fMGf0e07c3ALv-YmVKw";
        let aud = "https://todos.example.com/";
        let verifier = JwtVerifier::new("http://localhost:1234");
        let resp = verifier.verify::<Claims>(jwt, aud).await;
        println!("{:#?}", resp);
        assert!(resp.is_err());
        assert_eq!(resp.unwrap_err().to_string(), "ExpiredSignature");
    }

    #[tokio::test]
    async fn test_jwt_verifier_with_cache() {
        // Set up the mock
        let _m = mock("GET", "/.well-known/jwks.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"keys":[{"kty":"RSA","use":"sig","n":"7Z89Y4HjYOWQlePNfPFAiL24SG9GdPtiPF6SjQVe5X26KNQrpT0vBGGsfixbQ5NoBpXviFk8qHXi1cdyBwqr8eve8hEo9Kw91_NTco1BM2hIs3kSttfvRKg9ySfV0T4c0kuDdVVlZSNh2l1jOHqeM5oYhL-Ujq9jIG-JAy63WZx_lmsQN_5adHgNBT54YgEW9oNBl4MTSeFbA1ffDrXbW0OtqktiveCHQGI17_eE-RytNZ5PwCL2D793lNDf3sRNY4r4_VVDrF84En3Jr_rY6ogzxN3LSw43ewFOP0igRps4ZmVrzHvqrjbHn8in0sO6mICwsaBthn4oF92AtKDoKw","e":"AQAB","kid":"1zu17SECvh_Zcg4s9QPqX","x5t":"Vx_J2QjyEk-0NXQvF-thh29n6Q8","x5c":["MIIDHTCCAgWgAwIBAgIJOV8w2KgE5VN5MA0GCSqGSIb3DQEBCwUAMCwxKjAoBgNVBAMTIWRldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbTAeFw0yMjExMjMxMzI1NThaFw0zNjA4MDExMzI1NThaMCwxKjAoBgNVBAMTIWRldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbTCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAO2fPWOB42DlkJXjzXzxQIi9uEhvRnT7Yjxeko0FXuV9uijUK6U9LwRhrH4sW0OTaAaV74hZPKh14tXHcgcKq/Hr3vIRKPSsPdfzU3KNQTNoSLN5ErbX70SoPckn1dE+HNJLg3VVZWUjYdpdYzh6njOaGIS/lI6vYyBviQMut1mcf5ZrEDf+WnR4DQU+eGIBFvaDQZeDE0nhWwNX3w6121tDrapLYr3gh0BiNe/3hPkcrTWeT8Ai9g+/d5TQ397ETWOK+P1VQ6xfOBJ9ya/62OqIM8Tdy0sON3sBTj9IoEabOGZla8x76q42x5/Ip9LDupiAsLGgbYZ+KBfdgLSg6CsCAwEAAaNCMEAwDwYDVR0TAQH/BAUwAwEB/zAdBgNVHQ4EFgQUbMuvaPAXW0x0UIs2PQRrjN4mvJIwDgYDVR0PAQH/BAQDAgKEMA0GCSqGSIb3DQEBCwUAA4IBAQAHUIHuNR309kVV5vDCBIOr/NqmACT1ADh83cGMjc2KfYmdWt0iaR2QQdToXSZx8y6QKeGaZ77696na0OdYDkf/ngYX7YovhgsDgy65h+c2o+myIgeViWIZvqCt7+v+7kCw1DNkEhwYQx7/4DWf91uOqQmDGkrEFbk2h/2e0TmhYFgg9isBQ0+lWdL2kutdaoC+a+I3krIdLKqHgqQbs+d57y4/h6rHmZMv55WGXvKN21wu6JcMmzFkB1GNrJ/Ce7nIWRa0Kz5RVn4Yuq6BK18yTFI3w227i1Jz440Ce4eumQ0zsaEl+ZYNcJ9MU5sqUI3gji582nIkWHf42A692ZTC"],"alg":"RS256"},{"kty":"RSA","use":"sig","n":"xDG7pvlsuNrJ4AkOs2MZY9zpw4Qlqqbg5pXUhPbu33ahl27WU8M1zzkbne2i6_aHV71NcHp_C_OYzvo9-zw-AWHKj6UTp6JXca5MJJcE3djiHVbyCz0Du2MWQX_YDZb_2LncjbmnSbmIgN83k5vntBg-k4bJHR7RBkm5GDR7rSEUxGfJ7lOFgKY5HI4xIluk6u6YZ91GQK1BFi3kk_tBysyHZQMHp3A_vf584uYV42Kz6pJb-ZAZ94ZdIvxOUENSgEGwaA3qS1F8yByNg6n9axlTaN37XU8NBu4nld4w5XdTrvRyIxVrz8MfXRl6ILup1pNMeupx4SKlH_6i64juMw","e":"AQAB","kid":"v8NYxpog922LekQ_geMou","x5t":"Fy6Iq7McnGKDrlvwm2xpan4qOAo","x5c":["MIIDHTCCAgWgAwIBAgIJOHyUS8nhvDq/MA0GCSqGSIb3DQEBCwUAMCwxKjAoBgNVBAMTIWRldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbTAeFw0yMjExMjMxMzI1NThaFw0zNjA4MDExMzI1NThaMCwxKjAoBgNVBAMTIWRldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbTCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAMQxu6b5bLjayeAJDrNjGWPc6cOEJaqm4OaV1IT27t92oZdu1lPDNc85G53touv2h1e9TXB6fwvzmM76Pfs8PgFhyo+lE6eiV3GuTCSXBN3Y4h1W8gs9A7tjFkF/2A2W/9i53I25p0m5iIDfN5Ob57QYPpOGyR0e0QZJuRg0e60hFMRnye5ThYCmORyOMSJbpOrumGfdRkCtQRYt5JP7QcrMh2UDB6dwP73+fOLmFeNis+qSW/mQGfeGXSL8TlBDUoBBsGgN6ktRfMgcjYOp/WsZU2jd+11PDQbuJ5XeMOV3U670ciMVa8/DH10ZeiC7qdaTTHrqceEipR/+ouuI7jMCAwEAAaNCMEAwDwYDVR0TAQH/BAUwAwEB/zAdBgNVHQ4EFgQU3C3hbxhquy/RGdSdUy0pe/pRSXAwDgYDVR0PAQH/BAQDAgKEMA0GCSqGSIb3DQEBCwUAA4IBAQBwpMJXoTmkqkLogUgjXKP2V3bj8A9BUlZ3HWazblEIhjqXE84BwFdYLOozTsVPaUEjeGilRq28sBt/qkPCkZRi4JSd4Kiuri69NfYSPgW1rZrVBpkHylPwp0XNkBnu5xczU5184/3VNgv2czOsmWj4EP0OgBGHwTXB9/POQPP11rUzz0N/sv7uv4xrnAov5W/33alVm9GKga958/S75fUantzq6vBBLhmbWuwnqCE6o6a4axpU7HA67B6+QSoxZcHauq2rdbJgtksEfGGitBY5lle25SOKAZ+tHj0ZJnm5dx6etOhhk1k96sr8fP7qpOkgEXOJLZ0fvr6Pj+U12w6K"],"alg":"RS256"}]}"#)
            .create();

        let jwt = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjF6dTE3U0VDdmhfWmNnNHM5UVBxWCJ9.eyJpc3MiOiJodHRwczovL2Rldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbS8iLCJzdWIiOiJhdXRoMHw2NTEyY2U1MzUxODYwNDlmYjJhOTAxODEiLCJhdWQiOlsiaHR0cHM6Ly90b2Rvcy5leGFtcGxlLmNvbS8iLCJodHRwczovL2Rldi1vZ282YWJtdzV4MGhzdWVyLnVzLmF1dGgwLmNvbS91c2VyaW5mbyJdLCJpYXQiOjE2OTY2Mzk5MjUsImV4cCI6MTY5NjcyNjMyNSwiYXpwIjoiRlFRTjJRVmRobldQb1M3eFZqOGp2SnZTWU1oSDNYVVQiLCJzY29wZSI6Im9wZW5pZCBwcm9maWxlIGVtYWlsIG9mZmxpbmVfYWNjZXNzIn0.Q65UjlmbHHcDL7WIHTQ30Zy6PFi46bfxaJBu8pxcRtUiQzWugj6kkwt9FsCyStCJhahcWIZDfrtHBaweH3ynkS4n05HXYBtuUAK-hbWgR-NcXY31z9HdiSjY67gpYUoLvbuwytSlmh7rryN80jUR9HpivKtfN9i-6A45gf1R14TzkPKxmvDLRIGHiSnlqM7WFitEUfRCkaRuV4SEVyGRpX4VHwVBq7e5m2SoEPuNOnRenl56VmROcJhXBwNvdBzqrYkWDDx_pvZbY0iPeFiUL3pVzdQh_PCHtWq25nNKGFGm3hxMPloNXkHsqncDgMl2y08fMGf0e07c3ALv-YmVKw";
        let aud = "https://todos.example.com/";
        let verifier = JwtVerifier::new("http://localhost:1234")
            .use_cache(true)
            .build();
        let resp = verifier.verify::<Claims>(jwt, aud).await;
        println!("{:#?}", resp);
        assert!(resp.is_err());
        assert_eq!(resp.unwrap_err().to_string(), "ExpiredSignature");
    }
}
