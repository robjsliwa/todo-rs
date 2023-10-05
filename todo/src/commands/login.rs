use crate::auth;
use crate::config::Config;

pub fn login(config: &Config) {
    auth::login(config)
}
