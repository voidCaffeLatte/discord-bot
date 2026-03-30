use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::value_object::twitch::clip::Clip;

#[async_trait]
pub trait ClipGateway {
    async fn get_by_broadcaster_id(
        &self,
        broadcaster_id: &str,
        at: &DateTime<Utc>,
    ) -> Result<Arc<Vec<Clip>>, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("retrieval failed")]
    RetrievalFailed(#[source] anyhow::Error),

    #[error("invalid response")]
    InvalidResponse(#[source] anyhow::Error),
}
