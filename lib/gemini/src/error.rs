#[derive(Debug, thiserror::Error)]
pub enum GeminiError {
    #[error("failed to send HTTP request to Gemini API")]
    HttpRequestFailed(#[source] anyhow::Error),

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

    #[error("failed to parse Gemini API response as JSON")]
    JsonParseError(#[source] anyhow::Error),

    #[error("failed to decode base64 image data from Gemini API response")]
    Base64DecodeError(#[source] anyhow::Error),
}
