#[derive(Debug, thiserror::Error)]
pub enum GeminiError {
    #[error(transparent)]
    HttpRequestFailed(#[from] reqwest::Error),

    #[error("API returned no candidates")]
    NoCandidates,

    #[error("candidate contained no text")]
    NoText,
}
