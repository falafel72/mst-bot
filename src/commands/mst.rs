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

    if delay_times.is_empty() {
        return "No data for this user found!".to_string();
    }

    let total_delay_sec: i64 = delay_times.iter().map(|t| t.delay_seconds).sum();
    let average_sec = total_delay_sec / (delay_times.len() as i64);

    let mins = average_sec / 60;
    let secs = average_sec % 60;
    let qualifier = if average_sec < 0 { "early" } else { "late" };

    format!(
        "{} is {} minutes and {} seconds {} on average!",
        user.name,
        mins.abs(),
        secs.abs(),
        qualifier
    )
}

pub fn register() -> CreateCommand {
    CreateCommand::new("mst")
        .description("Display a particular user's MST")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "user", "The user to inquire about")
                .required(true),
        )
}
