#[derive(Debug, thiserror::Error)]
pub enum GeminiError {
    #[error(transparent)]
    HttpRequestFailed(#[from] reqwest::Error),

    #[error("API returned HTTP {status}: {body}")]
    HttpStatusError {
        status: reqwest::StatusCode,
        body: String,
    },

    #[error("API returned no candidates")]
    NoCandidates,

    #[error("candidate contained no text")]
    NoText,

    #[error("candidate contained no image")]
    NoImage,

    #[error(transparent)]
    JsonParseError(#[from] serde_json::Error),

    #[error(transparent)]
    Base64DecodeError(#[from] base64::DecodeError),
}
