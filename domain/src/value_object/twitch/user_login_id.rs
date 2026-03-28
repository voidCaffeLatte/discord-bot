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
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("abc123")]
    #[case("user_name_1")]
    #[case("a")]
    fn try_new_should_accept_valid_id(#[case] input: &str) {
        let id = UserLoginId::try_new(input.to_string()).unwrap();
        assert_eq!(id.id(), input);
    }

    #[test]
    fn try_new_should_return_error_when_empty() {
        assert!(matches!(UserLoginId::try_new("".to_string()), Err(Error::Empty)));
    }

    #[rstest]
    #[case("あ")]
    #[case("!!!")]
    #[case("abc-def")]
    #[case("ABC")]
    #[case("hello world")]
    fn try_new_should_return_error_for_invalid_format(#[case] input: &str) {
        assert!(matches!(UserLoginId::try_new(input.to_string()), Err(Error::InvalidFormat)));
    }
}
