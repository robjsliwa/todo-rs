use crate::commands::invoke_command;

mod commands;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    invoke_command();

    Ok(())
}
