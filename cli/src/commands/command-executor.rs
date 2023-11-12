use super::CommandContext;
use cred_store::CredStore;

pub trait CommandExecutor<T: CredStore> {
    fn execute(&self, context: &mut CommandContext<T>);
}
