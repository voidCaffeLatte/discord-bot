use chrono::{DateTime, Datelike, Utc};

#[derive(Clone)]
pub struct ImageGenerationActivity {
    user_id: String,
    count: u32,
    last_generated_at: Option<DateTime<Utc>>,
}

impl ImageGenerationActivity {
    pub const MAX_COUNT_PER_DAY: u32 = 2;

    pub fn with_key(user_id: String) -> Self {
        Self {
            user_id,
            count: 0,
            last_generated_at: None,
        }
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn increment_count(&mut self, at: &DateTime<Utc>) {
        self.count = self.current_count(at) + 1;
        self.last_generated_at = Some(*at);
    }

    pub fn is_available(&self, at: &DateTime<Utc>) -> bool {
        self.current_count(at) < Self::MAX_COUNT_PER_DAY
    }

    fn current_count(&self, at: &DateTime<Utc>) -> u32 {
        let Some(last_generated_at) = self.last_generated_at else { return self.count; };

        let last_generated_at = last_generated_at.with_timezone(&chrono_tz::Asia::Tokyo);
        let at = at.with_timezone(&chrono_tz::Asia::Tokyo);
        if at > last_generated_at && at.day() != last_generated_at.day() { return 0; }

        self.count
    }
}
