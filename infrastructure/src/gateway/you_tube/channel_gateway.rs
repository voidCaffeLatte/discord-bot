use application;
use application::gateway::you_tube::channel_gateway::Error;
use domain::value_object::you_tube::channel::Channel;
use domain::value_object::you_tube::channel_handle::ChannelHandle;

pub struct ChannelGateway {
    you_tube_data_api_key: String,
    http_client: reqwest::Client,
}

impl ChannelGateway {
    const BASE_URL: &'static str = "https://www.googleapis.com/youtube/v3/channels";

    pub fn new(
        you_tube_data_api_key: String,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            you_tube_data_api_key,
            http_client,
        }
    }
}

#[async_trait::async_trait]
impl application::gateway::you_tube::channel_gateway::ChannelGateway for ChannelGateway {
    async fn get_by_handle(&self, handle: &ChannelHandle) -> Result<Channel, Error> {
        let response = self.http_client.get(Self::BASE_URL)
            .query(&[("key", self.you_tube_data_api_key.as_str()), ("part", "id,contentDetails,snippet"), ("forHandle", handle.handle())])
            .send()
            .await
            .map_err(|e| Error::RetrievalFailed(e.into()))?
            .error_for_status()
            .map_err(|e| Error::RetrievalFailed(e.into()))?
            .json::<dto::Response>()
            .await
            .map_err(|_error| Error::InvalidResponse)?;

        let item = response.items.as_ref()
            .filter(|items| !items.is_empty())
            .and_then(|items| items.first())
            .ok_or(Error::ChannelNotFound)?;

        let title = item.snippet.as_ref()
            .and_then(|snippet| snippet.title.clone())
            .unwrap_or_default();

        let uploaded_video_playlist_id = item.content_details.as_ref()
            .and_then(|content_details| content_details.related_playlists.as_ref())
            .map(|related_playlists| related_playlists.uploads.clone());

        Ok(Channel::new(title, uploaded_video_playlist_id))
    }
}

mod dto {
    #[derive(Debug, serde::Deserialize)]
    pub struct Response {
        pub items: Option<Vec<Item>>,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Item {
        // pub id: String,
        pub snippet: Option<Snippet>,
        pub content_details: Option<ContentDetails>,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Snippet {
        pub title: Option<String>,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ContentDetails {
        pub related_playlists: Option<RelatedPlaylists>,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RelatedPlaylists {
        pub uploads: String,
    }
}
