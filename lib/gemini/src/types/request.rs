use schemars::Schema;
use super::Content;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub contents: Vec<Content>,
    pub tools: Option<Vec<Tool>>,
    pub system_instruction: Option<Content>,
    pub generation_config: Option<GenerationConfig>,
}

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
