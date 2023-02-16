use serenity::{
    prelude::*,
    async_trait,
    http::Http,
    model::{
        prelude::*,
        gateway::Ready,
        webhook::Webhook
    }
};
use serde_json::Value;
use super::youtube::{self, UpcomingStream};

async fn build_embed_fake(streams: &[UpcomingStream]) -> Vec<Value> {
    streams.iter().map(|stream| {
        let embed = Embed::fake(|e| {
            e.title(&stream.title)
                .url(format!("https://www.youtube.com/watch?v={}", stream.id))
                .thumbnail(&stream.thumbnail_url)
                .field("開始時刻", &stream.start_time, false)
                .image(&stream.thumbnail_url)
        });
        embed
    }).collect()
}

async fn send_message(http: impl AsRef<Http>, webhook: &Webhook, embeds: Vec<Value>) {
    webhook.execute(&http, false, |w| w.content("").embeds(embeds))
            .await
            .expect("Could not execute webhook.");
}

async fn update_message(http: impl AsRef<Http>, webhook: &Webhook, message_id: MessageId, embeds: Vec<Value>) {
    webhook.edit_message(&http, message_id, |w| w.content("").embeds(embeds))
        .await
        .expect("Could not edit message.");
}

pub struct Handler {
    pub webhook: Webhook
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let hub = youtube::Hub::new().await;
        let Ok(v) = hub.upcoming_streams().await else { return; };
        let streams = build_embed_fake(&v).await;

        let message = &self.webhook.channel_id.unwrap()
            .messages(&ctx.http, |f| f.limit(1)).await.unwrap()[0];

        if message.webhook_id == Some(self.webhook.id) {
            update_message(&ctx.http, &self.webhook, message.id, streams).await;
        }
        else {
            send_message(&ctx.http, &self.webhook, streams).await;
        }
    }
}