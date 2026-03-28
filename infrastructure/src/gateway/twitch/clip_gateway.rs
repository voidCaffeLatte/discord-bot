use application::gateway::twitch::clip_gateway;
use crate::gateway::twitch;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use dashmap::DashMap;
use common::cached_value::CachedValue;
use domain::value_object::twitch::clip::Clip;

pub struct ClipGateway {
    http_client: Arc<reqwest::Client>,
    twitch_app_client_id: String,
    twitch_access_token_gateway: Arc<twitch::access_token_gateway::AccessTokenGateway>,
    cached_clips: DashMap<String, CachedValue<Arc<Vec<Clip>>>>,
}

impl ClipGateway {
    const BASE_URL: &'static str = "https://api.twitch.tv/helix/clips";
    const CLIP_COUNT_PER_FETCH: usize = 100;
    const FETCH_COUNT: usize = 25;
    const CACHE_AVAILABLE_HOURS: i64 = 24;

    pub fn new(
        http_client: Arc<reqwest::Client>,
        twitch_app_client_id: String,
        twitch_access_token_gateway: Arc<twitch::access_token_gateway::AccessTokenGateway>,
    ) -> Self {
        Self {
            http_client,
            twitch_app_client_id,
            twitch_access_token_gateway,
            cached_clips: DashMap::new(),
        }
    }
}

#[async_trait]
impl clip_gateway::ClipGateway for ClipGateway {
    async fn get_by_broadcaster_id(
        &self,
        broadcaster_id: &str,
        at: &DateTime<Utc>,
    ) -> Result<Arc<Vec<Clip>>, clip_gateway::Error> {
        if let Some(clips) = self.cached_clips.get(broadcaster_id).as_ref()
            .and_then(|clips| clips.available_value(at)) {
            return Ok(clips.clone());
        }

        let access_token = self.twitch_access_token_gateway.get_access_token(at).await?;

        let mut results: Vec<Clip> = Vec::with_capacity(Self::CLIP_COUNT_PER_FETCH * Self::FETCH_COUNT);
        let mut pagination_cursor: Option<String> = None;

        for _ in 0..Self::FETCH_COUNT {
            let mut query_parameters = HashMap::new();
            query_parameters.insert("broadcaster_id", broadcaster_id.to_string());
            query_parameters.insert("first", Self::CLIP_COUNT_PER_FETCH.to_string());
            if let Some(pagination_cursor) = pagination_cursor {
                query_parameters.insert("after", pagination_cursor);
            }

            let response = self.http_client
                .get(Self::BASE_URL)
                .bearer_auth(access_token.access_token())
                .header("Client-Id", &self.twitch_app_client_id)
                .query(&query_parameters)
                .send()
                .await
                .map_err(clip_gateway::Error::RetrievalFailed)?
                .json::<dto::Response>()
                .await
                .map_err(|_error| clip_gateway::Error::InvalidResponse)?;

            let raw_clips = response.data.unwrap_or_default();
            let clips = raw_clips.iter()
                .map(|value| value.try_into())
                .collect::<Result<Vec<Clip>, _>>()?;

            results.extend(clips);

            let next_pagination_cursor = response.pagination
                .and_then(|pagination| pagination.cursor.clone());
            let Some(next_pagination_cursor) = next_pagination_cursor else { break; };
            pagination_cursor = Some(next_pagination_cursor);

            tokio::time::sleep(Duration::from_millis(20)).await;
        }

        let results = Arc::new(results);

        let cache = CachedValue::new(
            results.clone(), *at, chrono::Duration::hours(Self::CACHE_AVAILABLE_HOURS),
        );
        self.cached_clips.insert(broadcaster_id.to_string(), cache);

        Ok(results)
    }
}

mod dto {
    use std::time::Duration;
    use chrono::{DateTime, Utc};
    use application::gateway::twitch::clip_gateway;
    use domain::value_object::twitch::clip::Clip;

    #[derive(Debug, serde::Deserialize)]
    pub struct Response {
        pub data: Option<Vec<Data>>,
        pub pagination: Option<Pagination>,
    }

    #[derive(Debug, serde::Deserialize)]
    pub struct Data {
        pub url: String,
        pub title: String,
        pub view_count: i32,
        pub created_at: String,
        pub duration: f32,
    }

    impl TryFrom<&Data> for Clip {
        type Error = clip_gateway::Error;

        fn try_from(value: &Data) -> Result<Self, Self::Error> {
            let created_at = DateTime::parse_from_rfc3339(&value.created_at)
                .map_err(|_error| clip_gateway::Error::InvalidResponse)?
                .with_timezone(&Utc);
            let duration = Duration::from_secs_f32(value.duration);
            Ok(Clip::new(value.url.clone(), value.title.clone(), value.view_count, created_at, duration))
        }
    }

    #[derive(Debug, serde::Deserialize)]
    pub struct Pagination {
        pub cursor: Option<String>,
    }
}

impl From<twitch::access_token_gateway::Error> for clip_gateway::Error {
    fn from(value: twitch::access_token_gateway::Error) -> Self {
        match value {
            twitch::access_token_gateway::Error::APIRequestFailed(error) => clip_gateway::Error::RetrievalFailed(error),
        }
    }
}
