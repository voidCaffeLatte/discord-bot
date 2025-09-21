use async_trait::async_trait;
use thiserror::Error;
use domain::value_object::ai_text::AIText;

#[async_trait]
pub trait AITextGenerationGateway {
    async fn generate_text(&self, messages: &[Message], system_instruction: &str) -> Result<AIText, GatewayError>;
}

#[derive(Debug)]
pub enum Role {
    User,
    Model,
}

#[derive(Debug)]
pub struct Message {
    role: Role,
    message: String,
}

impl Message {
    pub fn new(
        role: Role,
        message: String,
    ) -> Self
    {
        Self {
            role,
            message,
        }
    }

    pub fn role(&self) -> &Role {
        &self.role
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error(transparent)]
    APIRequestFailed(#[from] reqwest::Error),

    #[error("invalid response")]
    InvalidResponse,
}
