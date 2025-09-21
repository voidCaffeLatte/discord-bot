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
