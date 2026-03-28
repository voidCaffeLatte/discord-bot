#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PrivacyStatus {
    Public,
    Private,
    Unlisted,
    Unspecified,
}

impl PrivacyStatus {
    pub fn is_public(&self) -> bool {
        *self == Self::Public
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(PrivacyStatus::Public, true)]
    #[case(PrivacyStatus::Private, false)]
    #[case(PrivacyStatus::Unlisted, false)]
    #[case(PrivacyStatus::Unspecified, false)]
    fn is_public_should_return_expected(#[case] status: PrivacyStatus, #[case] expected: bool) {
        assert_eq!(status.is_public(), expected);
    }
}
