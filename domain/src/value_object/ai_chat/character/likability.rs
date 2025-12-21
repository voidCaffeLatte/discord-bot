#[derive(Clone, Debug)]
pub struct Likability {
    value: i32,
}

impl Likability {
    pub const BASE: i32 = 0;
    pub const MIN: i32 = -100;
    pub const MAX: i32 = 100;

    pub fn new(value: i32) -> Result<Self, Error> {
        if !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(Error::InvalidRange(Self::MIN, Self::MAX, value));
        }

        Ok(
            Self {
                value
            }
        )
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}

impl Default for Likability {
    fn default() -> Self {
        Likability { value: Self::BASE }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("value must be between {0} and {1} - length: {2}")]
    InvalidRange(i32, i32, i32)
}
