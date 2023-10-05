use crate::config::Config;

pub trait CommandExecutor {
    fn execute(&self, config: &Config);
}
