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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use rstest::rstest;

    #[rstest]
    #[case(13, 0, true)]  // after expiry
    #[case(12, 0, true)]  // exact expiry
    #[case(11, 0, false)] // before expiry
    fn is_expired_should_return_expected(#[case] hour: u32, #[case] min: u32, #[case] expected: bool) {
        let token = AccessToken::new(
            "token".to_string(),
            Utc.with_ymd_and_hms(2026, 3, 28, 12, 0, 0).unwrap(),
        );
        let at = Utc.with_ymd_and_hms(2026, 3, 28, hour, min, 0).unwrap();
        assert_eq!(token.is_expired(&at), expected);
    }
}
