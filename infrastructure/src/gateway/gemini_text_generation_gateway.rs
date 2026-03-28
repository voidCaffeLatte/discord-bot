use application::gateway::ai_text_generation_gateway::GatewayError;
use application::gateway::ai_text_generation_gateway::{AITextGenerationGateway, Message, Role};
use async_trait::async_trait;
use domain::value_object::ai_text::{AIText, WebReference};
use gemini::GeminiClient;
use schemars::Schema;

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
}

#[async_trait]
impl AITextGenerationGateway for GeminiTextGenerationGateway {
    async fn generate_text(
        &self,
        messages: &[Message],
        system_instruction: &str,
        response_schema: Option<Schema>,
    ) -> Result<AIText, GatewayError> {
        let request = gemini::types::Request {
            messages: messages.iter().map(|message| gemini::types::Message {
                role: match message.role() {
                    Role::User => gemini::types::Role::User,
                    Role::Model => gemini::types::Role::Model,
                },
                text: message.message().to_string(),
            }).collect(),
            system_instruction: Some(system_instruction.to_string()),
            tools: vec![gemini::types::Tool::GoogleSearch, gemini::types::Tool::UrlContext],
        };

        let response = self.client.generate_content(&request).await
            .map_err(|error| match error {
                gemini::GeminiError::HttpRequestFailed(error) => GatewayError::APIRequestFailed(error),
                _ => GatewayError::InvalidResponse,
            })?;

        let web_references = response.grounding
            .map(|grounding| {
                grounding.chunks.iter()
                    .map(|chunk| WebReference::new(chunk.title.clone(), chunk.uri.clone()))
                    .collect()
            })
            .filter(|web_references: &Vec<WebReference>| !web_references.is_empty());

        Ok(AIText::new(response.text, web_references))
    }
}
