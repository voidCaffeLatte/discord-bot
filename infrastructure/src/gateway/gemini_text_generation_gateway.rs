use application::gateway::ai_text_generation_gateway::GatewayError;
use application::gateway::ai_text_generation_gateway::{AITextGenerationGateway, Message, Role};
use async_trait::async_trait;
use domain::value_object::ai_text::{AIText, WebReference};
use gemini::GeminiClient;
use gemini::types as gemini_types;
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
        let messages = messages.iter().map(
            |message| {
                let role = match message.role()
                {
                    Role::User => "user",
                    Role::Model => "model"
                };
                let part = gemini_types::Part { text: Some(message.message().to_string()) };

                gemini_types::Content {
                    parts: Some(vec![part]),
                    role: Some(role.to_string()),
                }
            })
            .collect::<Vec<_>>();

        let system_message = gemini_types::Content {
            parts: Some(vec![gemini_types::Part { text: Some(system_instruction.into()) }]),
            role: None,
        };

        let tools = vec![
            gemini_types::Tool { google_search: Some(gemini_types::GoogleSearch {}), url_context: None },
            gemini_types::Tool { google_search: None, url_context: Some(gemini_types::UrlContext {}) },
        ];

        let generation_config = response_schema.as_ref().map(|schema| {
            gemini_types::GenerationConfig {
                response_mime_type: Some("application/json".to_string()),
                response_json_schema: Some(schema.clone()),
            }
        });

        let request = gemini_types::Request {
            contents: messages,
            tools: Some(tools),
            system_instruction: Some(system_message),
            generation_config,
        };

        let response = self.client.generate_content(&request).await
            .map_err(|e| match e {
                gemini::GeminiError::HttpRequestFailed(e) => GatewayError::APIRequestFailed(e),
            })?;

        let candidate = response.candidates.as_ref()
            .and_then(|candidates| candidates.first())
            .ok_or(GatewayError::InvalidResponse)?;

        let parts = candidate.content.as_ref()
            .and_then(|content| content.parts.as_ref())
            .ok_or(GatewayError::InvalidResponse)?;
        let text = parts.iter()
            .filter_map(|part| part.text.as_deref())
            .collect::<Vec<_>>()
            .join("\n");

        // TODO: Refactoring
        let web_references = candidate.grounding_metadata.as_ref()
            .and_then(|grounding_metadata| grounding_metadata.grounding_chunks.as_ref())
            .map(|chunks| {
                chunks.iter()
                    .filter_map(|chunk| chunk.web.as_ref())
                    .map(|web| {
                        WebReference::new(
                            web.title.clone().unwrap_or_default(),
                            web.uri.clone().unwrap_or_default(),
                        )
                    })
                    .collect()
            })
            .filter(|web_reference: &Vec<WebReference>| !web_reference.is_empty());
        let ai_text = AIText::new(
            text,
            web_references,
        );

        Ok(ai_text)
    }
}
