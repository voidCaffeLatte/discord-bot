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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use rstest::rstest;

    fn tokyo(month: u32, day: u32, hour: u32, min: u32) -> DateTime<Utc> {
        chrono_tz::Asia::Tokyo
            .with_ymd_and_hms(2026, month, day, hour, min, 0)
            .unwrap()
            .with_timezone(&Utc)
    }

    // current_chat_count returns 0 for a fresh activity
    #[test]
    fn current_chat_count_should_return_zero_when_no_activity() {
        let activity = AIChatActivity::with_key("user1".to_string());
        assert_eq!(activity.current_chat_count(&tokyo(3, 28, 12, 0)), 0);
    }

    // Querying later on the same Tokyo day preserves count; next day resets to 0
    #[rstest]
    #[case(tokyo(3, 28, 23, 59), 1)] // same Tokyo day, later
    #[case(tokyo(3, 29, 0, 1), 0)]   // next Tokyo day, resets
    fn current_chat_count_across_tokyo_midnight(#[case] query_at: DateTime<Utc>, #[case] expected: u32) {
        let mut activity = AIChatActivity::with_key("user1".to_string());
        activity.increment_chat_count(&tokyo(3, 28, 15, 0));
        assert_eq!(activity.current_chat_count(&query_at), expected);
    }

    // When at <= last_chatted_at (equal or earlier), count is never reset
    #[rstest]
    #[case(tokyo(3, 28, 12, 0))] // same timestamp
    #[case(tokyo(3, 27, 12, 0))] // earlier timestamp (different day, but at < last)
    fn current_chat_count_should_not_reset_when_at_is_not_after_last(#[case] query_at: DateTime<Utc>) {
        let mut activity = AIChatActivity::with_key("user1".to_string());
        activity.increment_chat_count(&tokyo(3, 28, 12, 0));
        assert_eq!(activity.current_chat_count(&query_at), 1);
    }

    #[test]
    fn increment_chat_count_should_accumulate_on_same_day() {
        let mut activity = AIChatActivity::with_key("user1".to_string());
        activity.increment_chat_count(&tokyo(3, 28, 10, 0));
        activity.increment_chat_count(&tokyo(3, 28, 11, 0));
        activity.increment_chat_count(&tokyo(3, 28, 12, 0));
        assert_eq!(activity.current_chat_count(&tokyo(3, 28, 12, 0)), 3);
    }

    #[test]
    fn increment_chat_count_should_reset_then_increment_on_new_day() {
        let mut activity = AIChatActivity::with_key("user1".to_string());
        activity.increment_chat_count(&tokyo(3, 28, 12, 0));
        activity.increment_chat_count(&tokyo(3, 28, 13, 0));

        activity.increment_chat_count(&tokyo(3, 29, 12, 0));
        assert_eq!(activity.current_chat_count(&tokyo(3, 29, 12, 0)), 1);
    }
}
