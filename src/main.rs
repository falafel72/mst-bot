mod commands;

use std::env;

use serenity::{all::{GatewayIntents, GuildId, Interaction}, Client, async_trait, client::{EventHandler, Context}, builder::{CreateInteractionResponse, CreateInteractionResponseMessage}};
use serenity::model::gateway::Ready;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
       if let Interaction::Command(command) = interaction {
           println!("Received command interaction: {command:#?}");

           let content = match command.data.name.as_str() {
               "meetup" => Some(commands::meetup::run(&command.data.options())),
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

    let mut client = Client::builder(&token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
