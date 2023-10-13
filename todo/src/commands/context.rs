use crate::config::Config;
use cred_store::CredStore;

#[derive(Debug)]
pub struct CommandContext<'a, T: CredStore> {
    pub config: &'a Config,
    pub cred_store: &'a mut T,
}
