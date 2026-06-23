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
        let Some(last_generated_at) = self.last_generated_at else {
            return self.count;
        };

        let last_generated_at = last_generated_at.with_timezone(&chrono_tz::Asia::Tokyo);
        let at = at.with_timezone(&chrono_tz::Asia::Tokyo);
        if at > last_generated_at && at.day() != last_generated_at.day() {
            return 0;
        }

        self.count
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

    // is_available transitions: available (0) -> available (1) -> unavailable (2)
    #[rstest]
    #[case(0, true)]
    #[case(1, true)]
    #[case(2, false)]
    fn is_available_should_reflect_daily_limit(#[case] increments: u32, #[case] expected: bool) {
        let mut activity = ImageGenerationActivity::with_key("user1".to_string());
        let at = tokyo(3, 28, 12, 0);
        for _ in 0..increments {
            activity.increment_count(&at);
        }
        assert_eq!(activity.is_available(&at), expected);
    }

    #[test]
    fn is_available_should_reset_after_tokyo_day_change() {
        let mut activity = ImageGenerationActivity::with_key("user1".to_string());
        let day1 = tokyo(3, 28, 12, 0);
        activity.increment_count(&day1);
        activity.increment_count(&day1);
        assert!(!activity.is_available(&day1));

        let day2 = tokyo(3, 29, 12, 0);
        assert!(activity.is_available(&day2));
    }

    #[test]
    fn increment_count_should_reset_then_increment_on_new_day() {
        let mut activity = ImageGenerationActivity::with_key("user1".to_string());
        let day1 = tokyo(3, 28, 12, 0);
        activity.increment_count(&day1);
        activity.increment_count(&day1);

        let day2 = tokyo(3, 29, 12, 0);
        activity.increment_count(&day2);
        // reset to 0 then incremented to 1 -- still available
        assert!(activity.is_available(&day2));
    }
}
