use super::CommandContext;

pub fn logout(context: &mut CommandContext) {
    if context.cred_store.delete().is_err() {
        println!("No credentials found.");
        return;
    }

    println!("Logged out.");
}
