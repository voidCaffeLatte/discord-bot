use crate::gateway::you_tube::channel_gateway::ChannelGateway;
use crate::gateway::you_tube::playlist_item_gateway::PlaylistItemGateway;
use crate::gateway::you_tube::video_gateway::VideoGateway;
use crate::gateway::you_tube::{channel_gateway, video_gateway};
use chrono::{DateTime, Utc};
use domain::value_object::you_tube::channel::Channel;
use domain::value_object::you_tube::channel_handle::ChannelHandle;
use domain::value_object::you_tube::video::Video;
use rand::prelude::IteratorRandom;
use std::sync::Arc;

pub struct GetRandomYouTubeVideoUseCase {
    you_tube_channel_gateway: Arc<dyn ChannelGateway + Send + Sync>,
    you_tube_playlist_item_gateway: Arc<dyn PlaylistItemGateway + Send + Sync>,
    you_tube_video_gateway: Arc<dyn VideoGateway + Send + Sync>,
}

impl GetRandomYouTubeVideoUseCase {
    pub fn new(
        you_tube_channel_gateway: Arc<dyn ChannelGateway + Send + Sync>,
        you_tube_playlist_item_gateway: Arc<dyn PlaylistItemGateway + Send + Sync>,
        you_tube_video_gateway: Arc<dyn VideoGateway + Send + Sync>,
    ) -> Self {
        Self {
            you_tube_channel_gateway,
            you_tube_playlist_item_gateway,
            you_tube_video_gateway,
        }
    }

    pub async fn run(
        &self,
        channel_handle: &ChannelHandle,
        at: &DateTime<Utc>,
    ) -> Result<Output, Error> {
        let channel = match self
            .you_tube_channel_gateway
            .get_by_handle(channel_handle)
            .await
        {
            Ok(channel) => channel,
            Err(channel_gateway::Error::ChannelNotFound) => return Err(Error::ChannelNotFound),
            Err(error) => return Err(Error::VideoRetrievalFailed(error.into())),
        };

        let uploaded_video_playlist_id = channel
            .uploaded_video_playlist_id()
            .ok_or(Error::VideoNotFound)?;

        let playlist_items = match self
            .you_tube_playlist_item_gateway
            .get_by_playlist_id(uploaded_video_playlist_id, at)
            .await
        {
            Ok(playlist_items) => playlist_items,
            Err(error) => return Err(Error::VideoRetrievalFailed(error.into())),
        };

        let public_playlist_items = playlist_items
            .iter()
            .filter(|playlist_item| playlist_item.privacy_status().is_public());

        let random_playlist_item = public_playlist_items
            .choose_stable(&mut rand::rng())
            .ok_or(Error::VideoNotFound)?;

        let video = match self
            .you_tube_video_gateway
            .get_by_video_id(random_playlist_item.video_id())
            .await
        {
            Ok(video) => video,
            Err(video_gateway::Error::VideoNotFound) => return Err(Error::VideoNotFound),
            Err(error) => return Err(Error::VideoRetrievalFailed(error.into())),
        };

        Ok(Output {
            you_tube_channel: channel,
            you_tube_video: video,
        })
    }
}

pub struct Output {
    pub you_tube_channel: Channel,
    pub you_tube_video: Video,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("channel is not found")]
    ChannelNotFound,

    #[error("video is not found")]
    VideoNotFound,

    #[error("failed to retrieve video")]
    VideoRetrievalFailed(#[source] anyhow::Error),
}
