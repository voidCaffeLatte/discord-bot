use chrono::{DateTime, Datelike, Utc};

#[derive(Clone)]
pub struct AIChatActivity {
    user_id: String,
    chat_count: u32,
    last_chatted_at: Option<DateTime<Utc>>,
}

impl AIChatActivity {
    pub fn with_key(user_id: String) -> Self {
        Self {
            user_id,
            chat_count: 0,
            last_chatted_at: None,
        }
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn increment_chat_count(&mut self, at: &DateTime<Utc>) {
        self.chat_count = self.current_chat_count(at) + 1;
        self.last_chatted_at = Some(*at);
    }

    pub fn current_chat_count(&self, at: &DateTime<Utc>) -> u32 {
        let Some(last_chatted_at) = self.last_chatted_at else { return self.chat_count; };

        let last_chatted_at = last_chatted_at.with_timezone(&chrono_tz::Asia::Tokyo);
        let at = at.with_timezone(&chrono_tz::Asia::Tokyo);
        if at > last_chatted_at && at.day() != last_chatted_at.day() { return 0; }

        self.chat_count
    }
}
