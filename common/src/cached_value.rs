use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone)]
pub struct CachedValue<T>
where
    T: Clone,
{
    value: T,
    cached_at: DateTime<Utc>,
    available_duration: Duration,
}

impl<T> CachedValue<T>
where
    T: Clone,
{
    pub fn new(value: T, at: DateTime<Utc>, available_duration: Duration) -> Self {
        Self {
            value,
            cached_at: at,
            available_duration,
        }
    }

    pub fn available_value(&self, at: &DateTime<Utc>) -> Option<&T> {
        if self.is_available(at) {
            Some(&self.value)
        } else {
            None
        }
    }

    fn is_available(&self, at: &DateTime<Utc>) -> bool {
        *at <= self.available_until()
    }

    fn available_until(&self) -> DateTime<Utc> {
        self.cached_at + self.available_duration
    }
}
