mod commands;

use std::env;

use chrono::Local;
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

        // todo: retrieve intended join time, calculate difference, and add database entry
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
       if let Interaction::Command(command) = interaction {
           println!("Received command interaction: {command:#?}");

           let content = match command.data.name.as_str() {
               "meetup" => Some(commands::meetup::run(&command.data.options(), &(self.database)).await),
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
               commands::meetup::register()
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
                .filename("database.sqlite")
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
