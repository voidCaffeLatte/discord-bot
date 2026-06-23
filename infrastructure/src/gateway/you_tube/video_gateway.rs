use application;
use application::gateway::you_tube::video_gateway::Error;
use domain::value_object::you_tube::video::Video;

pub struct VideoGateway {
    you_tube_data_api_key: String,
    http_client: reqwest::Client,
}

impl VideoGateway {
    const BASE_URL: &'static str = "https://www.googleapis.com/youtube/v3/videos";

    pub fn new(you_tube_data_api_key: String, http_client: reqwest::Client) -> Self {
        Self {
            you_tube_data_api_key,
            http_client,
        }
    }
}

#[async_trait::async_trait]
impl application::gateway::you_tube::video_gateway::VideoGateway for VideoGateway {
    async fn get_by_video_id(&self, video_id: &str) -> Result<Video, Error> {
        let queries = vec![
            ("key", self.you_tube_data_api_key.clone()),
            (
                "part",
                "id,snippet,status,contentDetails,statistics".to_string(),
            ),
            ("id", video_id.to_string()),
        ];

        let response = self
            .http_client
            .get(Self::BASE_URL)
            .query(&queries)
            .send()
            .await
            .map_err(|e| Error::RetrievalFailed(e.into()))?
            .error_for_status()
            .map_err(|e| Error::RetrievalFailed(e.into()))?
            .json::<dto::Response>()
            .await
            .map_err(|error| Error::InvalidResponse(error.into()))?;

        let item = response
            .items
            .as_ref()
            .and_then(|items| items.first())
            .ok_or(Error::VideoNotFound)?;

        Ok(item.try_into()?)
    }
}

mod dto {
    use application::gateway::you_tube::video_gateway;
    use chrono::{DateTime, Utc};
    use domain::value_object::you_tube::privacy_status::PrivacyStatus;
    use domain::value_object::you_tube::video::Video;
    use tracing::warn;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Response {
        pub items: Option<Vec<Item>>,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Item {
        pub id: String,
        pub snippet: Option<Snippet>,
        pub status: Option<Status>,
        pub content_details: Option<ContentDetails>,
        pub statistics: Option<Statistics>,
    }

    impl TryFrom<&Item> for Video {
        type Error = video_gateway::Error;

        fn try_from(value: &Item) -> Result<Self, Self::Error> {
            let privacy_status = value
                .status
                .as_ref()
                .and_then(|status| status.into())
                .unwrap_or_else(|| {
                    warn!("Failed to access to privacy status. Use \"Private\" instead.");
                    PrivacyStatus::Private
                });
            let title = value
                .snippet
                .as_ref()
                .and_then(|snippet| snippet.title.clone())
                .ok_or_else(|| {
                    Self::Error::InvalidResponse(anyhow::anyhow!(
                        "title field is missing in response"
                    ))
                })?;
            let view_count = value
                .statistics
                .as_ref()
                .and_then(|stats| stats.view_count.as_ref())
                .and_then(|view_count| view_count.parse::<u64>().ok())
                .ok_or_else(|| {
                    Self::Error::InvalidResponse(anyhow::anyhow!(
                        "view count field is missing in response"
                    ))
                })?;
            let published_at = value
                .snippet
                .as_ref()
                .and_then(|snippet| snippet.published_at.as_ref())
                .ok_or_else(|| {
                    Self::Error::InvalidResponse(anyhow::anyhow!(
                        "published_at field is missing in response"
                    ))
                })?;
            let published_at = DateTime::parse_from_rfc3339(published_at)
                .map_err(|error| Self::Error::InvalidResponse(error.into()))?
                .with_timezone(&Utc);
            let duration = value
                .content_details
                .as_ref()
                .and_then(|detail| detail.duration.as_ref())
                .ok_or_else(|| {
                    Self::Error::InvalidResponse(anyhow::anyhow!(
                        "duration field is missing in response"
                    ))
                })?;
            let duration = iso8601::duration(duration)
                .map_err(|error| Self::Error::InvalidResponse(anyhow::anyhow!("{error:?}")))?
                .into();

            Ok(Video::new(
                value.id.clone(),
                privacy_status,
                title,
                view_count,
                published_at,
                duration,
            ))
        }
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Snippet {
        pub published_at: Option<String>,
        pub title: Option<String>,
    }

    #[derive(Debug, serde::Deserialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct Status {
        pub privacy_status: Option<String>,
    }

    impl From<&Status> for Option<PrivacyStatus> {
        fn from(value: &Status) -> Self {
            value
                .privacy_status
                .as_ref()
                .and_then(|status| match status.as_str() {
                    "public" => Some(PrivacyStatus::Public),
                    "private" => Some(PrivacyStatus::Private),
                    "unlisted" => Some(PrivacyStatus::Unlisted),
                    "privacyStatusUnspecified" => Some(PrivacyStatus::Unspecified),
                    _ => None,
                })
        }
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ContentDetails {
        pub duration: Option<String>,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Statistics {
        pub view_count: Option<String>,
    }
}
