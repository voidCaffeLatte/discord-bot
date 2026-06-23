use serde_json::json;
use tracing::error;

pub struct WebhookLogger {
    http_client: reqwest::Client,
    url: String,
}

impl WebhookLogger {
    pub fn new(http_client: reqwest::Client, url: String) -> Self {
        Self { http_client, url }
    }

    pub async fn send(&self, message: &str) {
        let payload = json!({ "content": message });
        if let Err(error) = self.http_client.post(&self.url).json(&payload).send().await {
            error!("Failed to send log to webhook: {}", error);
        }
    }

    pub async fn report_error(&self, error: &anyhow::Error) {
        self.send(&format!("## ERROR\n```\n{:?}\n```", error)).await;
    }
}
