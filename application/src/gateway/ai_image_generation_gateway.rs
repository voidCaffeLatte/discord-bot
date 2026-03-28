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
    #[error("request failed")]
    RequestFailed(#[source] anyhow::Error),

    #[error("generation failed")]
    GenerationFailed,

    #[error("invalid response")]
    InvalidResponse(#[source] anyhow::Error),
}
