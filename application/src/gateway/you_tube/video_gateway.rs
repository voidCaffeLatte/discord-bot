use domain::value_object::you_tube::video::Video;

#[async_trait::async_trait]
pub trait VideoGateway {
    async fn get_by_video_id(
        &self,
        video_id: &str,
    ) -> Result<Video, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("retrieval failed")]
    RetrievalFailed(#[source] reqwest::Error),

    #[error("invalid response")]
    InvalidResponse,
    
    #[error("video not found")]
    VideoNotFound
}
