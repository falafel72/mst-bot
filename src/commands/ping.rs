use serenity::builder::CreateCommand;

pub fn run() -> String {
    "Pong!".to_string()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("A simple ping command.")
}
