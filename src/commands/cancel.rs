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
        return "Please provide a valid user!".to_string();
    };

    let user_id = user.id.get().to_string();

    let guild_id_str = guild_id.to_string();

    // remove the entry from the database
    let Ok(_) = sqlx::query!(
        "DELETE FROM meetups WHERE user_id = ? AND guild_id = ?",
        user_id,
        guild_id_str
    )
    .execute(database)
    .await
    else {
        return "No meetup with this user found in this server!".to_string();
    };

    format!("Canceled meetup with <@{}>", user.id.get())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("cancel")
        .description("Cancel a meet up")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::User,
                "user",
                "The user to cancel the meetup with",
            )
            .required(true),
        )
}
