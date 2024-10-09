mod commands;

use std::env;

use chrono::Local;
use serenity::model::application::Command;
use serenity::model::gateway::Ready;
use serenity::{
    all::{GatewayIntents, Interaction, VoiceState},
    async_trait,
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage},
    client::{Context, EventHandler},
    Client,
};

struct Bot {
    database: sqlx::SqlitePool,
}

#[async_trait]
impl EventHandler for Bot {
    async fn voice_state_update(&self, _ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        // if the old VoiceState is not None, we can ignore the event
        if old.is_some() {
            println!("User was in a voice chat already.");
            return;
        };

        let Some(member) = new.member else {
            println!("Member invalid!");
            return;
        };

        let Some(guild_id) = new.guild_id else {
            println!("Voice channel in a DM!");
            return;
        };

        let dt = Local::now();
        let user = member.user;
        println!("{} joined at {}", user.name, dt.to_rfc3339());
        let user_id = user.id.get().to_string();
        let guild_id_str = guild_id.get().to_string();

        let Ok(intended_join_time) = sqlx::query!(
            "SELECT datetime_unix FROM meetups WHERE user_id = ?",
            user_id
        )
        .fetch_one(&(self.database))
        .await
        else {
            println!("Entry for this user not found!");
            return;
        };

        let expected_timestamp = &intended_join_time.datetime_unix;
        let timestamp_diff = dt.timestamp() - expected_timestamp;
        println!("Timestamp difference: {timestamp_diff}");

        // remove the entry from the database
        sqlx::query!(
            "DELETE FROM meetups WHERE user_id = ? AND guild_id = ?",
            user_id,
            guild_id_str
        )
        .execute(&(self.database))
        .await
        .unwrap();

        sqlx::query!(
            "INSERT INTO delays (user_id, delay_seconds, guild_id) VALUES (?, ?, ?)",
            user_id,
            timestamp_diff,
            guild_id_str
        )
        .execute(&(self.database))
        .await
        .unwrap();
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("Received command interaction: {command:#?}");

            // extract the guild id, otherwise is a dm?
            let Some(guild_id) = command.guild_id else {
                println!("This command doesn't work in dms!");
                return;
            };

            let content = match command.data.name.as_str() {
                "meetup" => Some(
                    commands::meetup::run(
                        &command.data.options(),
                        &(self.database),
                        guild_id.get(),
                    )
                    .await,
                ),
                "mst" => Some(
                    commands::mst::run(&command.data.options(), &(self.database), guild_id.get())
                        .await,
                ),
                "estimate" => Some(
                    commands::estimate::run(
                        &command.data.options(),
                        &(self.database),
                        guild_id.get(),
                    )
                    .await,
                ),
                "cancel" => Some(
                    commands::cancel::run(
                        &command.data.options(),
                        &(self.database),
                        guild_id.get(),
                    )
                    .await,
                ),
                "ping" => Some(commands::ping::run()),
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let _ = Command::create_global_command(&ctx.http, commands::ping::register()).await;
        let _ = Command::create_global_command(&ctx.http, commands::meetup::register()).await;
        let _ = Command::create_global_command(&ctx.http, commands::mst::register()).await;
        let _ = Command::create_global_command(&ctx.http, commands::estimate::register()).await;
        let _ = Command::create_global_command(&ctx.http, commands::cancel::register()).await;
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;

    // create database
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("database.db")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");

    sqlx::migrate!("./migrations")
        .run(&database)
        .await
        .expect("Couldn't run database migrations");

    let bot = Bot { database };

    let mut client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
