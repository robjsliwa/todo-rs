use crate::config::Config;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct DeviceAuthResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: String,
    expires_in: usize,
    interval: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    token_type: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<usize>,
    scope: Option<String>,
}

pub fn login(config: &Config) {
    // Get Device Code
    let client = Client::new();
    let resp = client
        .post(&format!("https://{}/oauth/device/code", config.domain))
        .form(&[
            ("client_id", config.client_id.as_str()),
            ("audience", config.audience.as_str()),
            ("scope", "openid profile email offline_access"),
        ])
        .send();

    println!("{:#?}", resp);

    let response = match resp {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    let device_auth_response: DeviceAuthResponse = match response.json::<DeviceAuthResponse>() {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    println!(
        "Go to {} and enter the code: {}",
        device_auth_response.verification_uri, device_auth_response.user_code
    );

    open::that(device_auth_response.verification_uri_complete).unwrap();

    let token_endpoint = format!("https://{}/oauth/token", config.domain);

    // Polling for token.
    loop {
        println!("Polling for token...");
        let resp: TokenResponse = client
            .post(&token_endpoint)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", &device_auth_response.device_code),
                ("client_id", config.client_id.as_str()),
            ])
            .send()
            .unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            })
            .json::<TokenResponse>()
            .unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            });

        println!("poll resp {:?}", resp);
        if let Some(access_token) = resp.access_token {
            println!("Access Token: {}", access_token);
            break;
        }

        std::thread::sleep(std::time::Duration::from_secs(
            device_auth_response.interval as u64,
        ));
    }
}
