use super::Content;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub candidates: Option<Vec<Candidate>>,
    pub usage_metadata: Option<UsageMetadata>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub content: Option<Content>,
    pub finish_reason: Option<String>,
    pub safety_ratings: Option<Vec<SafetyRating>>,
    pub citation_metadata: Option<CitationMetadata>,
    pub token_count: Option<i32>,
    pub grounding_metadata: Option<GroundingMetadata>,
    pub url_context_metadata: Option<UrlContextMetadata>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyRating {
    pub category: String,
    pub probability: String,
    pub blocked: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationMetadata {
    pub citation_sources: Option<Vec<CitationSource>>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationSource {
    pub start_index: Option<i32>,
    pub end_index: Option<i32>,
    pub uri: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetadata {
    pub candidates_token_count: Option<i32>,
    pub total_token_count: Option<i32>,
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
    pub confidence_scores: Option<Vec<f32>>,
    pub grounding_chunk_indices: Option<Vec<i32>>,
    pub segment: Option<GroundingSupportSegment>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingSupportSegment {
    pub end_index: Option<i32>,
    pub part_index: Option<i32>,
    pub start_index: Option<i32>,
    pub text: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlContextMetadata {
    pub url_metadata: Option<Vec<UrlMetadata>>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlMetadata {
    pub retrieved_url: Option<String>,
    pub url_retrieval_status: Option<String>,
}
