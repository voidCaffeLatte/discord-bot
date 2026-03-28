use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait AIIMageGenerationGateway {
    async fn generate_image(&self, prompt: &str) -> Result<Image, GatewayError>;
}

pub struct Image {
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error(transparent)]
    APIRequestFailed(#[from] reqwest::Error),

    #[error("failed to generate images")]
    GenerationFailed,

    #[error(transparent)]
    ImageDecodingFailed(#[from] base64::DecodeError),
}
