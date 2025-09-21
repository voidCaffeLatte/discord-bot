use std::time::Duration;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Clip {
    url: String,
    title: String,
    view_count: i32,
    created_at: DateTime<Utc>,
    duration: Duration,
}

impl Clip {
    pub fn new(
        url: String,
        title: String,
        view_count: i32,
        created_at: DateTime<Utc>,
        duration: Duration,
    ) -> Self {
        Self {
            url: url.to_string(),
            title: title.to_string(),
            view_count,
            created_at,
            duration,
        }
    }

    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn view_count(&self) -> i32 {
        self.view_count
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn duration(&self) -> &Duration {
        &self.duration
    }
}