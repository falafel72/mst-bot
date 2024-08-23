use chrono::{DateTime, Duration};
use serenity::{builder::{CreateCommand, CreateCommandOption}, all::{CommandOptionType, ResolvedOption, ResolvedValue}};

pub async fn run(options: &[ResolvedOption<'_>], database: &sqlx::SqlitePool) -> String {
    let Some(ResolvedOption {
        value: ResolvedValue::User(user, _), ..
    }) = options.first() else {
        return "Please provide a valid user".to_string();
    };

    let user_id = user.id.get().to_string();

    let Ok(delay_times) = sqlx::query!(
        "SELECT * FROM delays WHERE user_id = ?",
        user_id
    )
    .fetch_all(database)
    .await else {
        return "No data for this user found!".to_string();
    };


    let total_delay_sec : i64 = delay_times.iter().map(|t| t.delay_seconds).sum();

    let Ok(meetup_entry) = sqlx::query!(
        "SELECT datetime FROM meetups WHERE user_id = ?",
        user_id
    )
    .fetch_one(database)
    .await else {
        return "This person does not currently have a meetup scheduled!".to_string();
    };

    let Ok(meetup_timestamp) = DateTime::parse_from_rfc3339(&meetup_entry.datetime) else {
        let db_dt = meetup_entry.datetime;
        return format!("Unable to parse datetime stored in database: {}", db_dt);
    };

    let Some(duration) = Duration::new(total_delay_sec, 0) else {
        return "Duration invalid".to_string();
    };
    let new_time = meetup_timestamp + duration;
    let new_time_str = new_time.format("%I:%M%p");

    format!("{} is estimated to arrive at {}.", user.name, new_time_str)
}

pub fn register() -> CreateCommand {
   CreateCommand::new("estimate")
        .description("Display the estimated time a person might arrive")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "user", "The user to inquire about")
                .required(true),
        )
}
