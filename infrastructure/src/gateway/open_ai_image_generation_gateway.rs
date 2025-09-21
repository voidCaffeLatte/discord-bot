use std::sync::Arc;
use async_trait::async_trait;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use serde_json::json;
use application::gateway::ai_image_generation_gateway::{AIIMageGenerationGateway, GatewayError, Image};

pub struct OpenAIImageGenerationGateway {
    api_key: String,
    http_client: Arc<reqwest::Client>,
}

impl OpenAIImageGenerationGateway {
    const BASE_URL: &'static str = "https://api.openai.com/v1/images/generations";

    pub fn new(
        api_key: String,
        http_client: Arc<reqwest::Client>,
    ) -> Self {
        Self {
            api_key,
            http_client,
        }
    }
}

#[async_trait]
impl AIIMageGenerationGateway for OpenAIImageGenerationGateway {
    async fn generate_image(&self, prompt: &str) -> Result<Image, GatewayError> {
        let prompt = format!("I NEED to test how the tool works with extremely simple prompts. DO NOT add any detail, just use it AS-IS: {}", prompt);

        let request_body = json!({
            "model": "dall-e-3",
            "prompt": prompt,
            "response_format": "b64_json",
        });

        let response = self.http_client
            .post(Self::BASE_URL)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await?
            .json::<response::Response>()
            .await?;

        let data = response.data.first().ok_or(GatewayError::GenerationFailed)?;

        let image_bytes = BASE64_STANDARD.decode(&data.b64_json)?;

        Ok(Image { bytes: image_bytes })
    }
}

mod response {
    #[derive(Debug, serde::Deserialize)]
    pub struct Response {
        pub data: Vec<Data>,
    }


    #[derive(Debug, serde::Deserialize)]
    pub struct Data {
        pub b64_json: String,
        // revised_prompt: Option<String>,
    }
}
