use std::sync::Arc;
use chrono::{DateTime, Utc};
use domain::value_object::you_tube::playlist_item::PlaylistItem;

#[async_trait::async_trait]
pub trait PlaylistItemGateway {
    async fn get_by_playlist_id(
        &self,
        playlist_id: &str,
        at: &DateTime<Utc>,
    ) -> Result<Arc<Vec<PlaylistItem>>, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("retrieval failed")]
    RetrievalFailed(#[source] anyhow::Error),

    #[error("invalid response")]
    InvalidResponse,
}
