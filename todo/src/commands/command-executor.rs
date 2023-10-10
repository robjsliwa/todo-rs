use super::CommandContext;

pub trait CommandExecutor {
    fn execute(&self, context: &mut CommandContext);
}
