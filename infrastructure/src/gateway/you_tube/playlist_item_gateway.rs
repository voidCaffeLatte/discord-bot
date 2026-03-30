use std::sync::Arc;
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use tracing::warn;
use domain::value_object::you_tube::playlist_item::PlaylistItem;
use domain::value_object::you_tube::privacy_status::PrivacyStatus;
use application;
use application::gateway::you_tube::playlist_item_gateway::Error;
use common::cached_value::CachedValue;

pub struct PlaylistItemGateway {
    you_tube_data_api_key: String,
    http_client: reqwest::Client,
    playlist_caches: DashMap<String, CachedValue<Arc<Vec<PlaylistItem>>>>,
}

impl PlaylistItemGateway {
    const BASE_URL: &'static str = "https://www.googleapis.com/youtube/v3/playlistItems";
    const VIDEO_COUNT_PER_FETCH: usize = 50;
    const FETCH_COUNT: usize = 50;
    const CACHE_AVAILABLE_HOURS: i64 = 24;

    pub fn new(
        you_tube_data_api_key: String,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            you_tube_data_api_key,
            http_client,
            playlist_caches: DashMap::default(),
        }
    }
}

#[async_trait::async_trait]
impl application::gateway::you_tube::playlist_item_gateway::PlaylistItemGateway for PlaylistItemGateway {
    async fn get_by_playlist_id(&self, playlist_id: &str, at: &DateTime<Utc>) -> Result<Arc<Vec<PlaylistItem>>, Error> {
        if let Some(videos) = self.playlist_caches.get(playlist_id).as_ref()
            .and_then(|cache_value| cache_value.available_value(at)) {
            return Ok(videos.clone());
        }

        let mut results: Vec<PlaylistItem> = Vec::with_capacity(Self::VIDEO_COUNT_PER_FETCH * Self::FETCH_COUNT);
        let mut page_token: Option<String> = None;
        for _ in 0..Self::FETCH_COUNT {
            let mut queries = vec![
                ("key", self.you_tube_data_api_key.clone()),
                ("part", "status,contentDetails".to_string()),
                ("maxResults", Self::VIDEO_COUNT_PER_FETCH.to_string()),
                ("playlistId", playlist_id.to_string())
            ];
            if let Some(page_token) = page_token { queries.push(("pageToken", page_token)) }

            let response = self.http_client.get(Self::BASE_URL)
                .query(&queries)
                .send()
                .await
                .map_err(|e| Error::RetrievalFailed(e.into()))?
                .error_for_status()
                .map_err(|e| Error::RetrievalFailed(e.into()))?
                .json::<dto::Response>()
                .await
                .map_err(|error| Error::InvalidResponse(error.into()))?;

            let Some(items) = response.items else { break; };
            let items = items
                .iter()
                .map(|item| {
                    let privacy_status = item.status.privacy_status().unwrap_or_else(||
                        {
                            warn!("Unsupported privacy type: {}. Use \"Private\" instead.", item.status.privacy_status);
                            PrivacyStatus::Private
                        });
                    PlaylistItem::new(
                        item.content_details.video_id.clone(),
                        privacy_status,
                    )
                });

            results.extend(items);

            let Some(next_page_token) = response.next_page_token else { break; };
            page_token = Some(next_page_token);

            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }

        let results = Arc::new(results);

        let cache = CachedValue::new(results.clone(), *at, Duration::hours(Self::CACHE_AVAILABLE_HOURS));
        self.playlist_caches.insert(playlist_id.to_string(), cache);

        Ok(results)
    }
}


mod dto {
    use domain::value_object::you_tube::privacy_status::PrivacyStatus;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Response {
        pub items: Option<Vec<Item>>,
        pub next_page_token: Option<String>,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Item {
        pub status: Status,
        pub content_details: ContentDetails,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Status {
        pub privacy_status: String,
    }

    impl Status {
        pub fn privacy_status(&self) -> Option<PrivacyStatus> {
            match self.privacy_status.as_str() {
                "public" => Some(PrivacyStatus::Public),
                "private" => Some(PrivacyStatus::Private),
                "unlisted" => Some(PrivacyStatus::Unlisted),
                "privacyStatusUnspecified" => Some(PrivacyStatus::Unspecified),
                _ => None
            }
        }
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ContentDetails {
        pub video_id: String,
    }
}
