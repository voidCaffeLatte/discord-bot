use crate::dto;
use crate::error::GeminiError;
use crate::types;

pub struct GeminiClient {
    api_key: String,
    http_client: reqwest::Client,
    model_name: String,
}

impl GeminiClient {
    pub fn new(api_key: String, http_client: reqwest::Client, model_name: String) -> Self {
        Self {
            api_key,
            http_client,
            model_name,
        }
    }

    pub async fn generate_image(
        &self,
        prompt: &str,
    ) -> Result<types::ImageResponse, GeminiError> {
        let generation_config = dto::GenerationConfig {
            response_mime_type: None,
            response_json_schema: None,
            response_modalities: Some(vec!["IMAGE".to_string()]),
        };

        let dto_request = dto::Request {
            contents: vec![dto::Content {
                parts: Some(vec![dto::Part {
                    text: Some(prompt.to_string()),
                    inline_data: None,
                }]),
                role: Some("user".to_string()),
            }],
            tools: None,
            system_instruction: None,
            generation_config: Some(generation_config),
        };

        let dto_response = self.send_request(&dto_request).await?;
        let bytes = self.extract_image_response(&dto_response)?;

        Ok(types::ImageResponse { bytes })
    }

    fn api_url(&self) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.model_name
        )
    }

    pub async fn generate_content(
        &self,
        request: &types::Request,
    ) -> Result<types::Response, GeminiError> {
        let dto_request = self.build_dto_request(request, None);
        let dto_response = self.send_request(&dto_request).await?;
        let (text, grounding) = self.extract_response(&dto_response)?;

        Ok(types::Response { text, grounding })
    }

    pub async fn generate_content_structured<T>(
        &self,
        request: &types::Request,
    ) -> Result<types::StructuredResponse<T>, GeminiError>
    where
        T: schemars::JsonSchema + serde::de::DeserializeOwned,
    {
        let generation_config = dto::GenerationConfig {
            response_mime_type: Some("application/json".to_string()),
            response_json_schema: Some(schemars::schema_for!(T)),
            response_modalities: None,
        };
        let dto_request = self.build_dto_request(request, Some(generation_config));
        let dto_response = self.send_request(&dto_request).await?;
        let (text, grounding) = self.extract_response(&dto_response)?;
        let data: T = serde_json::from_str(&text)?;

        Ok(types::StructuredResponse { data, grounding })
    }

    async fn send_request(
        &self,
        request: &dto::Request,
    ) -> Result<dto::Response, GeminiError> {
        let response = self
            .http_client
            .post(self.api_url())
            .header("x-goog-api-key", &self.api_key)
            .json(request)
            .send()
            .await?
            .json::<dto::Response>()
            .await?;

        Ok(response)
    }

    fn build_dto_request(
        &self,
        request: &types::Request,
        generation_config: Option<dto::GenerationConfig>,
    ) -> dto::Request {
        let contents = request
            .messages
            .iter()
            .map(|message| {
                let role = match message.role {
                    types::Role::User => "user",
                    types::Role::Model => "model",
                };
                dto::Content {
                    parts: Some(vec![dto::Part {
                        text: Some(message.text.clone()),
                        inline_data: None,
                    }]),
                    role: Some(role.to_string()),
                }
            })
            .collect();

        let system_instruction = request.system_instruction.as_ref().map(|text| dto::Content {
            parts: Some(vec![dto::Part {
                text: Some(text.clone()),
                inline_data: None,
            }]),
            role: None,
        });

        let tools = request
            .tools
            .iter()
            .map(|tool| match tool {
                types::Tool::GoogleSearch => dto::Tool {
                    google_search: Some(dto::GoogleSearch {}),
                    url_context: None,
                },
                types::Tool::UrlContext => dto::Tool {
                    google_search: None,
                    url_context: Some(dto::UrlContext {}),
                },
            })
            .collect::<Vec<_>>();

        dto::Request {
            contents,
            tools: if tools.is_empty() { None } else { Some(tools) },
            system_instruction,
            generation_config,
        }
    }

    fn extract_response(
        &self,
        response: &dto::Response,
    ) -> Result<(String, Option<types::Grounding>), GeminiError> {
        let candidate = response
            .candidates
            .as_ref()
            .and_then(|candidates| candidates.first())
            .ok_or(GeminiError::NoCandidates)?;

        let text = candidate
            .content
            .as_ref()
            .and_then(|content| content.parts.as_ref())
            .and_then(|parts| {
                let texts: Vec<&str> = parts.iter().filter_map(|part| part.text.as_deref()).collect();
                if texts.is_empty() {
                    None
                } else {
                    Some(texts.join("\n"))
                }
            })
            .ok_or(GeminiError::NoText)?;

        let grounding = candidate.grounding_metadata.as_ref().and_then(|metadata| {
            let chunks: Vec<types::GroundingChunk> = metadata
                .grounding_chunks
                .as_ref()
                .map(|chunks| {
                    chunks
                        .iter()
                        .filter_map(|chunk| chunk.web.as_ref())
                        .map(|web| types::GroundingChunk {
                            title: web.title.clone().unwrap_or_default(),
                            uri: web.uri.clone().unwrap_or_default(),
                        })
                        .collect()
                })
                .unwrap_or_default();

            let web_search_queries = metadata
                .web_search_queries
                .clone()
                .unwrap_or_default();

            if chunks.is_empty() && web_search_queries.is_empty() {
                None
            } else {
                Some(types::Grounding {
                    chunks,
                    web_search_queries,
                })
            }
        });

        Ok((text, grounding))
    }

    fn extract_image_response(
        &self,
        response: &dto::Response,
    ) -> Result<Vec<u8>, GeminiError> {
        use base64::prelude::BASE64_STANDARD;
        use base64::Engine;

        let candidate = response
            .candidates
            .as_ref()
            .and_then(|candidates| candidates.first())
            .ok_or(GeminiError::NoCandidates)?;

        let inline_data = candidate
            .content
            .as_ref()
            .and_then(|content| content.parts.as_ref())
            .and_then(|parts| parts.iter().find_map(|part| part.inline_data.as_ref()))
            .ok_or(GeminiError::NoImage)?;

        let bytes = BASE64_STANDARD.decode(&inline_data.data)?;
        Ok(bytes)
    }
}
