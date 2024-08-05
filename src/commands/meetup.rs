use serenity::{builder::CreateCommand, all::ResolvedOption};

pub fn run(_options: &[ResolvedOption]) -> String {
    "Hey, I'm alive!".to_string()
}

pub fn register() -> CreateCommand {
   CreateCommand::new("meetup").description("Define a time to meet up")
}
