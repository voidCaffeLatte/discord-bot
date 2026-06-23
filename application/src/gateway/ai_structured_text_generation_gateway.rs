use crate::gateway::ai_text_generation_gateway::{GatewayError, Message};
use async_trait::async_trait;
use domain::value_object::ai_text::WebReference;

#[async_trait]
pub trait AIStructuredTextGenerationGateway<T: Send + Sync> {
    async fn generate_structured_text(
        &self,
        messages: &[Message],
        system_instruction: &str,
    ) -> Result<StructuredAIText<T>, GatewayError>;
}

pub struct StructuredAIText<T> {
    data: T,
    web_references: Option<Vec<WebReference>>,
}

impl<T> StructuredAIText<T> {
    pub fn new(data: T, web_references: Option<Vec<WebReference>>) -> Self {
        Self {
            data,
            web_references,
        }
    }

    pub fn into_parts(self) -> (T, Option<Vec<WebReference>>) {
        (self.data, self.web_references)
    }
}
