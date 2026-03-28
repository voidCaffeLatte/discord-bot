// ── Request ──

#[derive(Debug)]
pub enum Role {
    User,
    Model,
}

#[derive(Debug)]
pub struct Message {
    pub role: Role,
    pub text: String,
}

#[derive(Debug)]
pub enum Tool {
    GoogleSearch,
    UrlContext,
}

#[derive(Debug)]
pub struct TextRequest {
    pub messages: Vec<Message>,
    pub system_instruction: Option<String>,
    pub tools: Vec<Tool>,
}

// ── Response ──

#[derive(Debug)]
pub struct ImageResponse {
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

#[derive(Debug)]
pub struct TextResponse {
    pub text: String,
    pub grounding: Option<Grounding>,
}

#[derive(Debug)]
pub struct StructuredTextResponse<T> {
    pub data: T,
    pub grounding: Option<Grounding>,
}

#[derive(Debug)]
pub struct Grounding {
    pub chunks: Vec<GroundingChunk>,
    pub web_search_queries: Vec<String>,
}

#[derive(Debug)]
pub struct GroundingChunk {
    pub title: String,
    pub uri: String,
}
