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
        let response = self
            .http_client
            .post(self.api_url())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header("x-goog-api-key", &self.api_key)
            .body(serde_json::to_string(request).unwrap())
            .send()
            .await?
            .json::<types::Response>()
            .await?;

        Ok(response)
    }
}
