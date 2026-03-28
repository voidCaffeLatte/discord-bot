use application::gateway::ai_structured_text_generation_gateway::{AIStructuredTextGenerationGateway, StructuredAIText};
use application::gateway::ai_text_generation_gateway::GatewayError;
use application::gateway::ai_text_generation_gateway::{AITextGenerationGateway, Message, Role};
use async_trait::async_trait;
use domain::value_object::ai_text::{AIText, WebReference};
use gemini::GeminiClient;

pub struct GeminiTextGenerationGateway {
    client: GeminiClient,
}

impl GeminiTextGenerationGateway {
    pub fn new(
        api_key: String, http_client: reqwest::Client, model_name: String,
    ) -> Self {
        Self {
            client: GeminiClient::new(api_key, http_client, model_name),
        }
    }

    fn build_request(messages: &[Message], system_instruction: &str) -> gemini::types::Request {
        gemini::types::Request {
            messages: messages.iter().map(|message| gemini::types::Message {
                role: match message.role() {
                    Role::User => gemini::types::Role::User,
                    Role::Model => gemini::types::Role::Model,
                },
                text: message.message().to_string(),
            }).collect(),
            system_instruction: Some(system_instruction.to_string()),
            tools: vec![gemini::types::Tool::GoogleSearch, gemini::types::Tool::UrlContext],
        }
    }

    fn extract_web_references(grounding: Option<gemini::types::Grounding>) -> Option<Vec<WebReference>> {
        grounding
            .map(|grounding| {
                grounding.chunks.iter()
                    .map(|chunk| WebReference::new(chunk.title.clone(), chunk.uri.clone()))
                    .collect()
            })
            .filter(|web_references: &Vec<WebReference>| !web_references.is_empty())
    }

    fn map_error(error: gemini::GeminiError) -> GatewayError {
        match error {
            gemini::GeminiError::HttpRequestFailed(error) => GatewayError::APIRequestFailed(error),
            _ => GatewayError::InvalidResponse,
        }
    }
}

#[async_trait]
impl AITextGenerationGateway for GeminiTextGenerationGateway {
    async fn generate_text(
        &self,
        messages: &[Message],
        system_instruction: &str,
    ) -> Result<AIText, GatewayError> {
        let request = Self::build_request(messages, system_instruction);
        let response = self.client.generate_content(&request).await.map_err(Self::map_error)?;
        let web_references = Self::extract_web_references(response.grounding);

        Ok(AIText::new(response.text, web_references))
    }
}

#[async_trait]
impl<T> AIStructuredTextGenerationGateway<T> for GeminiTextGenerationGateway
where
    T: schemars::JsonSchema + serde::de::DeserializeOwned + Send + Sync,
{
    async fn generate_structured_text(
        &self,
        messages: &[Message],
        system_instruction: &str,
    ) -> Result<StructuredAIText<T>, GatewayError> {
        let request = Self::build_request(messages, system_instruction);
        let response = self.client.generate_content_structured::<T>(&request).await.map_err(Self::map_error)?;
        let web_references = Self::extract_web_references(response.grounding);

        Ok(StructuredAIText::new(response.data, web_references))
    }
}
