use application::gateway::ai_image_generation_gateway::{AIIMageGenerationGateway, GatewayError, Image};
use async_trait::async_trait;
use gemini::GeminiClient;

pub struct GeminiImageGenerationGateway {
    client: GeminiClient,
}

impl GeminiImageGenerationGateway {
    pub fn new(api_key: String, http_client: reqwest::Client, model_name: String) -> Self {
        Self {
            client: GeminiClient::new(api_key, http_client, model_name),
        }
    }

    fn map_error(error: gemini::GeminiError) -> GatewayError {
        match error {
            gemini::GeminiError::HttpRequestFailed(error) => GatewayError::APIRequestFailed(error),
            gemini::GeminiError::Base64DecodeError(error) => {
                GatewayError::ImageDecodingFailed(error)
            }
            _ => GatewayError::GenerationFailed,
        }
    }
}

#[async_trait]
impl AIIMageGenerationGateway for GeminiImageGenerationGateway {
    async fn generate_image(&self, prompt: &str) -> Result<Image, GatewayError> {
        let response = self
            .client
            .generate_image(prompt)
            .await
            .map_err(Self::map_error)?;

        Ok(Image {
            bytes: response.bytes,
            mime_type: response.mime_type,
        })
    }
}
