use super::token_response::TokenResponse;
use crate::config::Config;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize)]
struct DeviceAuthResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: String,
    expires_in: usize,
    interval: usize,
}

pub fn login(config: &Config) -> Result<TokenResponse, Box<dyn std::error::Error>> {
    let client = Client::new();
    let resp = client
        .post(&format!("https://{}/oauth/device/code", config.domain))
        .form(&[
            ("client_id", config.client_id.as_str()),
            ("audience", config.audience.as_str()),
            ("scope", "openid profile email offline_access"),
        ])
        .send();

    let response = match resp {
        Ok(resp) => resp,
        Err(e) => return Err(e.into()),
    };
    let device_auth_response: DeviceAuthResponse = match response.json::<DeviceAuthResponse>() {
        Ok(resp) => resp,
        Err(e) => return Err(e.into()),
    };

    println!(
        "Go to {} and enter the code: {}",
        device_auth_response.verification_uri, device_auth_response.user_code
    );

    _ = open::that(device_auth_response.verification_uri_complete);

    let token_endpoint = format!("https://{}/oauth/token", config.domain);

    let start_instant = Instant::now();
    let expiry_duration = Duration::from_secs(device_auth_response.expires_in as u64);

    let mut sp = Spinner::new(Spinners::Dots9, "Polling for token".into());

    loop {
        if Instant::now() >= start_instant + expiry_duration {
            sp.stop();
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Device code has expired",
            )));
        }

        let resp_result = client
            .post(&token_endpoint)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", &device_auth_response.device_code),
                ("client_id", config.client_id.as_str()),
            ])
            .send()
            .and_then(|res| res.json::<TokenResponse>());

        match resp_result {
            Ok(resp) => {
                if resp.access_token.is_some() {
                    sp.stop();
                    return Ok(resp);
                }
            }
            Err(e) => {
                sp.stop();
                return Err(Box::new(e));
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(
            device_auth_response.interval as u64,
        ));
    }
}
