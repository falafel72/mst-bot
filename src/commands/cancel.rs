use serenity::{
    all::{CommandOptionType, ResolvedOption, ResolvedValue},
    builder::{CreateCommand, CreateCommandOption},
};

pub async fn run(options: &[ResolvedOption<'_>], database: &sqlx::SqlitePool) -> String {
    let Some(ResolvedOption {
        value: ResolvedValue::User(user, _),
        ..
    }) = options.first()
    else {
        return "Please provide a valid user".to_string();
    };

    let user_id = user.id.get().to_string();

    // remove the entry from the database
    sqlx::query!("DELETE FROM meetups WHERE user_id = ?", user_id)
        .execute(database)
        .await
        .unwrap();

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
