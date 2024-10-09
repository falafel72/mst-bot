use chrono::{Local, NaiveDateTime, NaiveTime};
use serenity::{
    all::{CommandOptionType, ResolvedOption, ResolvedValue},
    builder::{CreateCommand, CreateCommandOption},
};

pub async fn run(
    options: &[ResolvedOption<'_>],
    database: &sqlx::SqlitePool,
    guild_id: u64,
) -> String {
    let Some(ResolvedOption {
        value: ResolvedValue::User(user, _),
        ..
    }) = options.first()
    else {
        return "Please provide a time string".to_string();
    };

    let Some(ResolvedOption {
        value: ResolvedValue::String(timestr),
        ..
    }) = options.get(1)
    else {
        return "Please provide a valid user".to_string();
    };

    // time formatting goes here
    let now = Local::now();
    let today = now.date_naive();
    let Ok(time) = NaiveTime::parse_from_str(timestr, "%H:%M") else {
        return "Invalid time string".to_string();
    };

    let Some(dt) = NaiveDateTime::new(today, time)
        .and_local_timezone(Local)
        .single()
    else {
        return "Unable to create datetime".to_string();
    };

    // save the user id timestamp pair to a database
    let user_str = user.id.get().to_string();
    let timestamp = dt.timestamp();
    let guild_id_str = guild_id.to_string();
    sqlx::query!(
        "INSERT INTO meetups (user_id, datetime_unix, guild_id) VALUES (?, ?, ?)",
        user_str,
        timestamp,
        guild_id_str
    )
    .execute(database)
    .await
    .unwrap();

    let timestamp = dt.timestamp();

    format!(
        "Meetup scheduled for <@{}> at <t:{}:t>",
        user_str, timestamp
    )
}

pub fn register() -> CreateCommand {
    CreateCommand::new("meetup")
        .description("Define a time to meet up")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "user", "the user to meet up with")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "time",
                "time to meet up, must be today in 24 hour time",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "date",
                "date to meet up, in format YYYY-MM-DD. Defaults to today's date.",
            )
            .required(false),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "timezone",
                "desired timezone for this time (will be today's date)",
            )
            .required(false),
        )
}
