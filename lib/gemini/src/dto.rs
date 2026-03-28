use schemars::Schema;

// ── Shared ──

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct Content {
    pub parts: Option<Vec<Part>>,
    pub role: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct Part {
    pub text: Option<String>,
}

// ── Request ──

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Request {
    pub contents: Vec<Content>,
    pub tools: Option<Vec<Tool>>,
    pub system_instruction: Option<Content>,
    pub generation_config: Option<GenerationConfig>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Tool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search: Option<GoogleSearch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_context: Option<UrlContext>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GoogleSearch {}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UrlContext {}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GenerationConfig {
    pub response_mime_type: Option<String>,
    pub response_json_schema: Option<Schema>,
}

// ── Response ──

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Response {
    pub candidates: Option<Vec<Candidate>>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Candidate {
    pub content: Option<Content>,
    pub grounding_metadata: Option<GroundingMetadata>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GroundingMetadata {
    pub grounding_chunks: Option<Vec<GroundingChunk>>,
    pub web_search_queries: Option<Vec<String>>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GroundingChunk {
    pub web: Option<GroundingChunkWeb>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GroundingChunkWeb {
    pub title: Option<String>,
    pub uri: Option<String>,
}
