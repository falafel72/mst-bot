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
        return "Please provide a valid user".to_string();
    };

    let user_id = user.id.get().to_string();

    let guild_id_str = guild_id.to_string();

    // we check if a guild id exists or is null to preserve backwards compatibility
    let Ok(delay_times) = sqlx::query!(
        "SELECT * FROM delays WHERE user_id = ? AND (guild_id = ? OR guild_id IS NULL)",
        user_id,
        guild_id_str
    )
    .fetch_all(database)
    .await
    else {
        return "No data for this user found!".to_string();
    };

    let total_delay_sec: i64 = delay_times.iter().map(|t| t.delay_seconds).sum();

    let Ok(meetup_entry) = sqlx::query!(
        "SELECT datetime_unix FROM meetups WHERE user_id = ? AND guild_id = ?",
        user_id,
        guild_id_str
    )
    .fetch_one(database)
    .await
    else {
        return "This person does not currently have a meetup scheduled!".to_string();
    };

    let meetup_timestamp = &meetup_entry.datetime_unix;
    let new_timestamp = meetup_timestamp + total_delay_sec;

    format!(
        "{} is estimated to arrive at <t:{}:t>.",
        user.name, new_timestamp
    )
}

pub fn register() -> CreateCommand {
    CreateCommand::new("estimate")
        .description("Display the estimated time a person might arrive")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "user", "The user to inquire about")
                .required(true),
        )
}
