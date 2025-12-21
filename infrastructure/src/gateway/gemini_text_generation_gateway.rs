use application::gateway::ai_text_generation_gateway::GatewayError;
use application::gateway::ai_text_generation_gateway::{AITextGenerationGateway, Message, Role};
use async_trait::async_trait;
use domain::value_object::ai_text::{AIText, WebReference};
use schemars::Schema;

pub struct GeminiTextGenerationGateway {
    api_key: String,
    http_client: reqwest::Client,
    model_name: String,
}

impl GeminiTextGenerationGateway {
    pub fn new(
        api_key: String, http_client: reqwest::Client, model_name: String,
    ) -> Self {
        Self {
            api_key,
            http_client,
            model_name,
        }
    }

    fn api_url(&self) -> String {
        format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", self.model_name)
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
                let part = dto::Part { text: Some(message.message().to_string()) };

                dto::Content {
                    parts: Some(vec![part]),
                    role: Some(role.to_string()),
                }
            })
            .collect::<Vec<_>>();

        let system_message = dto::Content {
            parts: Some(vec![dto::Part { text: Some(system_instruction.into()) }]),
            role: None,
        };

        let tools = vec![
            dto::Tool { google_search: Some(dto::GoogleSearch {}), url_context: None },
            dto::Tool { google_search: None, url_context: Some(dto::UrlContext {}) },
        ];

        let generation_config = response_schema.as_ref().map(|schema| {
            dto::GenerationConfig {
                response_mime_type: Some("application/json".to_string()),
                response_json_schema: Some(schema.clone()),
            }
        });

        let request_message = dto::Request {
            contents: messages,
            tools: Some(tools),
            system_instruction: Some(system_message),
            generation_config,
        };

        let response = self.http_client.post(self.api_url())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header("x-goog-api-key", &self.api_key)
            .body(serde_json::to_string(&request_message).unwrap())
            .send()
            .await?
            .json::<dto::Response>()
            .await?;

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

mod dto {
    use schemars::Schema;

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Request {
        pub contents: Vec<Content>,
        pub tools: Option<Vec<Tool>>,
        pub system_instruction: Option<Content>,
        pub generation_config: Option<GenerationConfig>,
    }

    #[derive(Debug, serde::Deserialize)]
    pub struct Response {
        pub candidates: Option<Vec<Candidate>>,
        // #[serde(alias = "usageMetadata")]
        // pub usage_metadata: Option<UsageMetadata>
    }


    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Candidate {
        pub content: Option<Content>,
        // #[serde(alias = "finishReason")]
        // pub finish_reason: Option<String>,
        // #[serde(alias = "safetyRatings")]
        // pub safety_ratings: Option<Vec<SafetyRating>>,
        // #[serde(alias = "citationMetadata")]
        // pub citation_metadata: Option<CitationMetadata>,
        // #[serde(alias = "tokenCount")]
        // pub token_count: Option<i32>,
        pub grounding_metadata: Option<GroundingMetadata>,
        // pub url_context_metadata: Option<UrlContextMetadata>,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct Content {
        pub parts: Option<Vec<Part>>,
        pub role: Option<String>,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct Part {
        pub text: Option<String>,
    }

    // #[derive(Debug, serde::Deserialize)]
    // pub struct SafetyRating {
    //     pub category: String,
    //     pub probability: String,
    //     pub blocked: bool,
    // }

    // #[derive(Debug, serde::Deserialize)]
    // pub struct CitationMetadata {
    //     #[serde(alias = "citationSources")]
    //     pub citation_sources: Option<Vec<CitationSource>>,
    // }

    // #[derive(Debug, serde::Deserialize)]
    // pub struct CitationSource {
    //     #[serde(alias = "startIndex")]
    //     pub start_index: Option<i32>,
    //     #[serde(alias = "endIndex")]
    //     pub end_index: Option<i32>,
    //     pub uri: Option<String>,
    //     pub license: Option<String>,
    // }

    // #[derive(Debug, serde::Deserialize)]
    // pub struct UsageMetadata {
    //     #[serde(alias = "candidatesTokenCount")]
    //     pub candidates_token_count: Option<i32>,
    //     #[serde(alias = "totalTokenCount")]
    //     pub total_token_count: Option<i32>,
    // }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Tool {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub google_search: Option<GoogleSearch>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub url_context: Option<UrlContext>,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GoogleSearch {}

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UrlContext {}

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GenerationConfig {
        pub response_mime_type: Option<String>,
        pub response_json_schema: Option<Schema>,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GroundingMetadata {
        pub grounding_chunks: Option<Vec<GroundingChunk>>,
        pub grounding_supports: Option<Vec<GroundingSupport>>,
        pub retrieval_queries: Option<Vec<String>>,
        pub web_search_queries: Option<Vec<String>>,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GroundingChunk {
        pub web: Option<GroundingChunkWeb>,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GroundingChunkWeb {
        pub title: Option<String>,
        pub uri: Option<String>,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GroundingSupport {
        confidence_scores: Option<Vec<f32>>,
        grounding_chunk_indices: Option<Vec<i32>>,
        segment: Option<GroundingSupportSegment>,
    }


    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GroundingSupportSegment {
        end_index: Option<i32>,
        part_index: Option<i32>,
        start_index: Option<i32>,
        text: Option<String>,
    }

    // #[derive(Debug, serde::Deserialize, serde::Serialize)]
    // #[serde(rename_all = "camelCase")]
    // pub struct UrlContextMetadata {
    //     url_metadata: Option<Vec<UrlMetadata>>,
    // }

    // #[derive(Debug, serde::Deserialize, serde::Serialize)]
    // #[serde(rename_all = "camelCase")]
    // pub struct UrlMetadata {
    //     pub retrieved_url: Option<String>,
    //     pub url_retrieval_status: Option<String>,
    // }
}
