#[derive(Clone, Debug)]
pub struct Channel {
    title: String,
    uploaded_video_playlist_id: Option<String>,
}

impl Channel {
    pub fn new(title: String, uploaded_video_playlist_id: Option<String>) -> Self {
        Self {
            title,
            uploaded_video_playlist_id,
        }
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn uploaded_video_playlist_id(&self) -> Option<&String> {
        self.uploaded_video_playlist_id.as_ref()
    }
}
