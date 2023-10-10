use crate::config::Config;
use cred_store::Credentials;

#[derive(Debug)]
pub struct CommandContext<'a> {
    pub config: &'a Config,
    pub cred_store: &'a mut Credentials,
}
