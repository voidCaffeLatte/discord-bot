use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct AccessToken {
    access_token: String,
    expires_at: DateTime<Utc>,
}

impl AccessToken {
    pub fn new(access_token: String, expires_at: DateTime<Utc>) -> Self {
        Self { access_token, expires_at }
    }

    pub fn is_expired(&self, at: &DateTime<Utc>) -> bool {
        self.expires_at <= *at
    }

    pub fn access_token(&self) -> &str {
        self.access_token.as_str()
    }
}
