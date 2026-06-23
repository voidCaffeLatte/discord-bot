use crate::value_object::you_tube::privacy_status::PrivacyStatus;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct Video {
    video_id: String,
    privacy_status: PrivacyStatus,
    title: String,
    view_count: u64,
    published_at: DateTime<Utc>,
    duration: std::time::Duration,
}

impl Video {
    pub fn new(
        video_id: String,
        privacy_status: PrivacyStatus,
        title: String,
        view_count: u64,
        published_at: DateTime<Utc>,
        duration: std::time::Duration,
    ) -> Self {
        Self {
            video_id: video_id.to_string(),
            privacy_status,
            title,
            view_count,
            published_at,
            duration,
        }
    }

    pub fn privacy_status(&self) -> &PrivacyStatus {
        &self.privacy_status
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn view_count(&self) -> u64 {
        self.view_count
    }

    pub fn published_at(&self) -> &DateTime<Utc> {
        &self.published_at
    }

    pub fn duration(&self) -> &std::time::Duration {
        &self.duration
    }

    pub fn video_url(&self) -> String {
        format!("https://www.youtube.com/watch?v={}", self.video_id)
    }
}
