use regex::Regex;

pub struct UserLoginId(String);

impl UserLoginId {
    pub fn try_new(id: String) -> Result<UserLoginId, Error> {
        if id.is_empty() {
            return Err(Error::Empty);
        }
        
        // TODO: Add length validation

        static REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new(r"^[a-z0-9_]+$").expect("Invalid regex pattern")
        });

        if !REGEX.is_match(id.as_str()) {
            return Err(Error::InvalidFormat);
        }

        Ok(UserLoginId(id))
    }

    pub fn id(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("user id is empty")]
    Empty,

    #[error("user id format is invalid")]
    InvalidFormat,
}

#[cfg(test)]
mod test {
    use super::*;

    mod try_new {
        use super::*;

        #[test]
        fn should_return_error_when_id_is_empty() {
            assert!(matches!(UserLoginId::try_new("".to_string()), Err(Error::Empty)));
        }

        #[test]
        fn should_return_error_when_id_format_is_invalid() {
            assert!(matches!(UserLoginId::try_new("あ".to_string()), Err(Error::InvalidFormat)));
            assert!(matches!(UserLoginId::try_new("!!!".to_string()), Err(Error::InvalidFormat)));
            assert!(matches!(UserLoginId::try_new("abc-def".to_string()), Err(Error::InvalidFormat)));
        }
    }
}
