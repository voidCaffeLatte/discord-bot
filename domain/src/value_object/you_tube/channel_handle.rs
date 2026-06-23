pub struct ChannelHandle(String);

impl ChannelHandle {
    pub const MIN_LENGTH: usize = 3;
    pub const MAX_LENGTH: usize = 30;

    pub fn try_new(handle: String) -> Result<Self, Error> {
        let length = handle.chars().count();
        if !(Self::MIN_LENGTH..=Self::MAX_LENGTH).contains(&length) {
            return Err(Error::InvalidLength(
                Self::MIN_LENGTH,
                Self::MAX_LENGTH,
                length,
            ));
        }

        Ok(Self(handle))
    }

    pub fn handle(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Handle length must be between {0} and {1} - length: {2}")]
    InvalidLength(usize, usize, usize),
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("abc")]
    #[case("あいう")]
    fn try_new_should_accept_at_min_length(#[case] input: &str) {
        let handle = ChannelHandle::try_new(input.to_string()).unwrap();
        assert_eq!(handle.handle(), input);
    }

    #[test]
    fn try_new_should_accept_at_max_length() {
        let input = "a".repeat(ChannelHandle::MAX_LENGTH);
        let handle = ChannelHandle::try_new(input.clone()).unwrap();
        assert_eq!(handle.handle(), input);
    }

    #[rstest]
    #[case("", 0)]
    #[case("aあ", 2)]
    #[case("aあaあaあaあaあaあaあaあaあaあaあaあaあaあaあa", 31)]
    fn try_new_should_reject_invalid_length(#[case] input: &str, #[case] expected_len: usize) {
        assert!(matches!(
            ChannelHandle::try_new(input.to_string()),
            Err(Error::InvalidLength(_, _, len)) if len == expected_len
        ));
    }
}
