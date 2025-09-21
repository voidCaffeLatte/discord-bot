use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::value_object::twitch::user::User;
use domain::value_object::twitch::user_login_id::UserLoginId;

#[async_trait]
pub trait UserGateway {
    async fn get_by_user_login_id(
        &self,
        user_login_id: &UserLoginId,
        at: &DateTime<Utc>,
    ) -> Result<User, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("retrieval failed")]
    RetrievalFailed(#[source] reqwest::Error),

    #[error("user is not found")]
    UserNotFound,

    #[error("invalid response")]
    InvalidResponse,
}
