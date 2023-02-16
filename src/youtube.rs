use anyhow::Result;
use base64::prelude::*;
use itertools::Itertools;
use google_youtube3 as youtube3;
use youtube3::{oauth2, hyper, hyper_rustls, YouTube};
use oauth2::ServiceAccountKey;
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use chrono::{DateTime, offset::Local};

pub struct Hub {
    hub: YouTube<HttpsConnector<HttpConnector>>
}

impl Hub {
    pub async fn new() -> Self {
        let secret_base64 = std::env::var("GOOGLE_SERVICE_ACCOUNT_KEY_BASE64").expect("");
        let secret = BASE64_STANDARD.decode(secret_base64).expect("");
        let key: ServiceAccountKey = serde_json::from_slice(&secret).expect("");
        let auth = oauth2::ServiceAccountAuthenticator::builder(key)
            .build()
            .await
            .unwrap();

        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();
        let client = hyper::Client::builder().build(connector);

        Self {
            hub: YouTube::new(client, auth)
        }
    }

    pub async fn upcoming_streams(&self) -> Result<Vec<UpcomingStream>> {
        let channel_id = std::env::var("YOUTUBE_CHANNEL_ID").expect("");
        
        let channel_icon_url = {
            let (_, res) = self.hub.channels().list(&vec!["snippet".into()])
                .add_id(&channel_id)
                .doit()
                .await?;
            let channel = &res.items.as_ref().unwrap()[0];
            let thumbnail = channel.snippet.as_ref().unwrap().thumbnails.as_ref().unwrap();
            let thumbnail_url = thumbnail.maxres.as_ref()
                .or(thumbnail.high.as_ref())
                .or(thumbnail.medium.as_ref())
                .or(thumbnail.default.as_ref())
                .or(thumbnail.standard.as_ref())
                .map(|e| e.url.clone().unwrap())
                .unwrap();
            thumbnail_url
        };

        let (_, response) = self.hub.search().list(&vec!["id".into(), "snippet".into()])
            .channel_id(&channel_id)
            .add_type("video")
            .event_type("upcoming")
            .doit()
            .await?;

        let start_times = {
            let video_ids = response.items.as_ref().unwrap().iter().map(|r| r.id.as_ref().unwrap().video_id.clone().unwrap()).join(",");
            let (_, response) = self.hub.videos().list(&vec!["liveStreamingDetails".into()])
                .add_id(&video_ids)
                .doit()
                .await?;
            response.items.as_ref().unwrap().iter().map(|v| {
                let time_str = v.live_streaming_details.as_ref().unwrap().scheduled_start_time.as_ref().unwrap();
                DateTime::parse_from_rfc3339(time_str)
                    .unwrap()
                    .with_timezone(&Local)
                    .format("%Y年%m月%d日 %H時%M分")
                    .to_string()
            }).collect_vec()
        };

        let streams = response.items.unwrap().iter().zip(start_times.into_iter()).map(|(r, start_time)| {
            let thumbnail = r.snippet.as_ref().unwrap().thumbnails.as_ref().unwrap();
            let thumbnail_url = thumbnail.maxres.as_ref()
                .or(thumbnail.high.as_ref())
                .or(thumbnail.medium.as_ref())
                .or(thumbnail.default.as_ref())
                .or(thumbnail.standard.as_ref())
                .map(|e| e.url.clone().unwrap())
                .unwrap();

            UpcomingStream {
                id: r.id.as_ref().unwrap().video_id.clone().unwrap(),
                title: r.snippet.as_ref().unwrap().title.clone().unwrap(),
                published_at: r.snippet.as_ref().unwrap().published_at.clone().unwrap(),
                thumbnail_url,
                start_time,
                channel_icon_url: channel_icon_url.clone()
            }
        }).collect();

        Ok(streams)
    }
}

#[derive(Debug, Clone)]
pub struct UpcomingStream {
    pub id: String,
    pub title: String,
    pub published_at: String,
    pub thumbnail_url: String,
    pub start_time: String,
    pub channel_icon_url: String
}