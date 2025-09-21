use crate::value_object::you_tube::privacy_status::PrivacyStatus;

#[derive(Clone, Debug)]
pub struct PlaylistItem {
    video_id: String,
    privacy_status: PrivacyStatus,
}

impl PlaylistItem {
    pub fn new(
        video_id: String,
        privacy_status: PrivacyStatus,
    ) -> Self {
        Self {
            video_id,
            privacy_status,
        }
    }

    pub fn video_id(&self) -> &str {
        &self.video_id
    }

    pub fn privacy_status(&self) -> &PrivacyStatus {
        &self.privacy_status
    }
}
