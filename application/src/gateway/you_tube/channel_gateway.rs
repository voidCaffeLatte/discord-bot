use async_trait::async_trait;
use domain::value_object::you_tube::channel::Channel;
use domain::value_object::you_tube::channel_handle::ChannelHandle;

#[async_trait]
pub trait ChannelGateway {
    async fn get_by_handle(
        &self,
        handle: &ChannelHandle,
    ) -> Result<Channel, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("retrieval failed")]
    RetrievalFailed(#[source] reqwest::Error),

    #[error("invalid response")]
    InvalidResponse,

    #[error("channel not found")]
    ChannelNotFound,
}
