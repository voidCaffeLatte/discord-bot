use crate::gateway::twitch;
use crate::gateway::twitch::clip_gateway::ClipGateway;
use crate::gateway::twitch::user_gateway::UserGateway;
use chrono::{DateTime, Utc};
use domain::value_object::twitch::clip::Clip;
use domain::value_object::twitch::user_login_id::UserLoginId;
use rand::prelude::IndexedRandom;
use std::sync::Arc;
use thiserror::Error;

pub struct GetRandomTwitchClipUseCase {
    twitch_clip_gateway: Arc<dyn ClipGateway + Send + Sync>,
    twitch_user_gateway: Arc<dyn UserGateway + Send + Sync>,
}

impl GetRandomTwitchClipUseCase {
    pub fn new(
        twitch_clip_gateway: Arc<dyn ClipGateway + Send + Sync>,
        twitch_user_gateway: Arc<dyn UserGateway + Send + Sync>,
    ) -> Self {
        Self {
            twitch_clip_gateway,
            twitch_user_gateway,
        }
    }

    pub async fn run(
        &self,
        user_login_id: &UserLoginId,
        amount: usize,
        at: &DateTime<Utc>,
    ) -> Result<Vec<Clip>, Error> {
        let twitch_user = self
            .twitch_user_gateway
            .get_by_user_login_id(user_login_id, at)
            .await
            .map_err(|error| match error {
                twitch::user_gateway::Error::UserNotFound => Error::UserNotFound,
                error => Error::TwitchUserAccessError(error.into()),
            })?;
        let clips = self
            .twitch_clip_gateway
            .get_by_broadcaster_id(twitch_user.id(), at)
            .await
            .map_err(|error| Error::TwitchClipAccessError(error.into()))?;
        Ok(Self::sample_random_clip(&clips, amount))
    }

    fn sample_random_clip(clips: &[Clip], amount: usize) -> Vec<Clip> {
        clips.sample(&mut rand::rng(), amount).cloned().collect()
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("target user is not found")]
    UserNotFound,

    #[error("failed to retrieve Twitch clips")]
    TwitchClipAccessError(#[source] anyhow::Error),

    #[error("twitch user access error")]
    TwitchUserAccessError(#[source] anyhow::Error),
}
