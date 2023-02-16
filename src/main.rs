mod discord;
mod youtube;

use serenity::{
    prelude::*,
    http::Http,
    model::webhook::Webhook
};
use discord::Handler;

#[tokio::main]
async fn main() {
    let url = std::env::var("DISCORD_WEBHOOK_URL").expect("");
    let http = Http::new("");
    let webhook = Webhook::from_url(&http, &url).await.expect("");

    let token = std::env::var("DISCORD_TOKEN").expect("");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler { webhook: webhook.clone() })
        .await
        .expect("Could not create client.");
    client.start().await.expect("Could not start client.");
}
