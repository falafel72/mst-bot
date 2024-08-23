use chrono::{NaiveDateTime, Local, NaiveTime};
use serenity::{builder::{CreateCommand, CreateCommandOption}, all::{ResolvedOption, CommandOptionType, ResolvedValue}};

pub async fn run(options: &[ResolvedOption<'_>], database: &sqlx::SqlitePool) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::User(user, _), ..
    }) = options.first() {
        if let Some(ResolvedOption{
            value: ResolvedValue::String(timestr), ..
        }) = options.get(1) {
            // time formatting goes here
            let now = Local::now();
            let today = now.date_naive();
            let Ok(time) = NaiveTime::parse_from_str(timestr, "%H:%M") else {
                return "Invalid time string".to_string()
            };

            let Some(dt) = NaiveDateTime::new(today, time).and_local_timezone(Local).single() else {
               return "Unable to create datetime".to_string()
            };

            // save the user id timestamp pair to a database
            let user_str = user.id.get().to_string();
            let datetime_str = dt.to_rfc3339();
            sqlx::query!(
                "INSERT INTO meetups (user_id, datetime) VALUES (?, ?)",
                user_str,
                datetime_str
            )
            .execute(database)
            .await
            .unwrap();

            format!("Meetup scheduled for <@{}> at {}", user_str, datetime_str)
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
