#[derive(Debug, thiserror::Error)]
pub enum GeminiError {
    #[error(transparent)]
    HttpRequestFailed(#[from] reqwest::Error),
}
