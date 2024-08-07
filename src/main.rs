mod commands;

use std::env;

use chrono::{Local, DateTime};
use serenity::{all::{GatewayIntents, GuildId, Interaction, VoiceState}, Client, async_trait, client::{EventHandler, Context}, builder::{CreateInteractionResponse, CreateInteractionResponseMessage}};
use serenity::model::gateway::Ready;

struct Bot {
    database: sqlx::SqlitePool,
}

#[async_trait]
impl EventHandler for Bot{
    async fn voice_state_update(&self,
                                _ctx: Context,
                                old: Option<VoiceState>,
                                new: VoiceState) {
        // if the old VoiceState is not None, we can ignore the event
        if old.is_some() {
            println!("User was in a voice chat already.");
            return;
        };

        let Some(member) = new.member else {
            println!("Member invalid!");
            return;
        };

        // ensure the timestamp is in utc format
        let timestamp = Local::now();

        let user = member.user;
        println!("{} joined at {}", user.name, timestamp.to_rfc3339());

        let user_id = user.id.get().to_string();

        let Ok(intended_join_time) = sqlx::query!(
            "SELECT datetime FROM meetups WHERE user_id = ?",
            user_id
        )
        .fetch_one(&(self.database))
        .await else {
            println!("Entry for this user not found!");
            return;
        };

        let Ok(expected_dt) = DateTime::parse_from_rfc3339(&intended_join_time.datetime) else {
            let db_dt = intended_join_time.datetime;
            println!("Unable to parse datetime stored in database: {db_dt}");
            return;
        };
        let timestamp_diff = timestamp.timestamp() - expected_dt.timestamp();
        println!("Timestamp difference: {timestamp_diff}");

        // remove the entry from the database
        sqlx::query!(
            "DELETE FROM meetups WHERE user_id = ?",
            user_id
        )
        .execute(&(self.database))
        .await
        .unwrap();

        sqlx::query!(
           "INSERT INTO delays (user_id, delay_seconds) VALUES (?, ?)",
            user_id,
            timestamp_diff
        )
        .execute(&(self.database))
        .await
        .unwrap();
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
       if let Interaction::Command(command) = interaction {
           println!("Received command interaction: {command:#?}");

           let content = match command.data.name.as_str() {
               "meetup" => Some(commands::meetup::run(&command.data.options(), &(self.database)).await),
               "mst" => Some(commands::mst::run(&command.data.options(), &(self.database)).await),
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

        let guild_id = GuildId::new(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = guild_id
            .set_commands(&ctx.http, vec![
                commands::meetup::register(),
                commands::mst::register()
            ])
            .await;

        println!("I have the following guild slash commands: {commands:#?}");
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
                .create_if_missing(true)
        )
        .await
        .expect("Couldn't connect to database");

    sqlx::migrate!("./migrations").run(&database).await.expect("Couldn't run database migrations");

    let bot = Bot {
        database,
    };

    let mut client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
