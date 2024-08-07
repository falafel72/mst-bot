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
    let average_sec = total_delay_sec / (delay_times.len() as i64);

    let mins = average_sec / 60;
    let secs = average_sec % 60;
    let qualifier = if average_sec < 0 { "early" } else { "late" };

    format!("{} is {} minutes and {} seconds {} on average!", user.name, mins.abs(), secs.abs(), qualifier)
}

pub fn register() -> CreateCommand {
   CreateCommand::new("mst")
        .description("Display a particular user's MST")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "user", "The user to inquire about")
                .required(true),
        )
}
