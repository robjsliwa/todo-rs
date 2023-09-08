use chrono::prelude::*;
use clap::Parser;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    secret: String,
    #[arg(short, long)]
    tenant_id: String,
    #[arg(short, long)]
    user_id: String,
    #[arg(short, long, default_value_t = 3600)]
    exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    tenant_id: String,
    user_id: String,
    exp: usize,
}

fn main() {
    let args = Args::parse();
    let tenant_id = args.tenant_id;
    let user_id = args.user_id;
    let duration = args.exp;

    let my_claims = Claims {
        tenant_id,
        user_id,
        exp: (Utc::now() + chrono::Duration::seconds(duration as i64)).timestamp() as usize,
    };

    let key = args.secret;
    let token = match encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(key.as_ref()),
    ) {
        Ok(t) => t,
        Err(_) => panic!("Error generating the token"),
    };
    println!("Generated JWT: {}", token);
}
