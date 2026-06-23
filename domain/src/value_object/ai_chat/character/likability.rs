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

        Ok(Self { value })
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
    InvalidRange(i32, i32, i32),
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, 0)]
    #[case(50, 50)]
    #[case(-100, -100)]
    #[case(100, 100)]
    fn new_should_create_with_valid_value(#[case] input: i32, #[case] expected: i32) {
        let likability = Likability::new(input).unwrap();
        assert_eq!(likability.value(), expected);
    }

    #[rstest]
    #[case(-101)]
    #[case(101)]
    #[case(i32::MIN)]
    #[case(i32::MAX)]
    fn new_should_reject_out_of_range(#[case] value: i32) {
        assert!(matches!(
            Likability::new(value),
            Err(Error::InvalidRange(_, _, _))
        ));
    }

    #[test]
    fn default_should_return_base_value() {
        assert_eq!(Likability::default().value(), Likability::BASE);
    }
}
