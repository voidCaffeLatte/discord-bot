pub struct ChannelHandle(String);

impl ChannelHandle {
    pub const MIN_LENGTH: usize = 3;
    pub const MAX_LENGTH: usize = 30;

    pub fn try_new(handle: String) -> Result<Self, Error> {
        let length = handle.chars().count();
        if !(Self::MIN_LENGTH..=Self::MAX_LENGTH).contains(&length) {
            return Err(Error::InvalidLength(Self::MIN_LENGTH, Self::MAX_LENGTH, length));
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
    InvalidLength(usize, usize, usize)
}

#[cfg(test)]
mod test {
    use super::*;

    mod try_new {
        use super::*;

        #[test]
        fn should_return_error_when_handle_length_is_invalid() {
            assert!(matches!(ChannelHandle::try_new("".to_string()), Err(Error::InvalidLength(_, _, 0))));
            assert!(matches!(ChannelHandle::try_new("aあ".to_string()), Err(Error::InvalidLength(_, _, 2))));
            assert!(matches!(ChannelHandle::try_new("aあaあaあaあaあaあaあaあaあaあaあaあaあaあaあa".to_string()), Err(Error::InvalidLength(_, _, 31))));
        }
    }
}
