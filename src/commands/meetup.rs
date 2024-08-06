use chrono::NaiveDateTime;
use serenity::{builder::{CreateCommand, CreateCommandOption}, all::{ResolvedOption, CommandOptionType, ResolvedValue}};

pub fn run(options: &[ResolvedOption]) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::User(user, _), ..
    }) = options.first() {
        if let Some(ResolvedOption{
            value: ResolvedValue::String(timestr), ..
        }) = options.get(1) {
            // time formatting goes here
            let _dt = NaiveDateTime::parse_from_str(timestr, "%H:%M");
            format!("Meeting with {} at {}", user.name, timestr)
        } else {
            "Please provide a time string".to_string()
        }
    } else {
        "Please provide a valid user".to_string()
    }
}

pub fn register() -> CreateCommand {
   CreateCommand::new("meetup")
        .description("Define a time to meet up")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "user", "the user to meet up with")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "time", "time to meet up, must be today in 24 hour time")
                .required(true),
        )
}
