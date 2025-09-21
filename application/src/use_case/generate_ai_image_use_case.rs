use crate::gateway::ai_image_generation_gateway::{AIIMageGenerationGateway, GatewayError};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use thiserror::Error;
use domain::model::image_generation_activity::ImageGenerationActivity;
use crate::repository::image_generation_activity_repository;
use crate::repository::image_generation_activity_repository::ImageGenerationActivityRepository;

pub struct GenerateAIImageUseCase {
    ai_image_generation_gateway: Arc<dyn AIIMageGenerationGateway + Send + Sync>,
    image_generation_activity_repository: Arc<dyn ImageGenerationActivityRepository + Send + Sync>,
}

impl GenerateAIImageUseCase {
    pub fn new(
        ai_image_generation_gateway: Arc<dyn AIIMageGenerationGateway + Send + Sync>,
        image_generation_activity_repository: Arc<dyn ImageGenerationActivityRepository + Send + Sync>,
    ) -> Self {
        Self {
            ai_image_generation_gateway,
            image_generation_activity_repository,
        }
    }

    pub async fn run(
        &self,
        user_id: impl Into<String>,
        prompt: impl Into<String>,
        at: &DateTime<Utc>,
    ) -> Result<UseCaseResult, UseCaseError> {
        let user_id = user_id.into();

        let mut image_generation_activity = match self.image_generation_activity_repository.get(&user_id) {
            Ok(image_generation_activity) => image_generation_activity,
            Err(image_generation_activity_repository::Error::EntryNotFound(_)) => ImageGenerationActivity::with_key(user_id.clone())
        };
        if !image_generation_activity.is_available(at) {
            return Err(UseCaseError::GenerationCountExceeded);
        }

        let prompt = prompt.into();

        let gateway_result = self.ai_image_generation_gateway.generate_image(&prompt).await?;

        image_generation_activity.increment_count(at);
        self.image_generation_activity_repository.set(image_generation_activity);

        Ok(UseCaseResult { image_bytes: gateway_result.bytes })
    }
}

pub struct UseCaseResult {
    pub image_bytes: Vec<u8>,
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("generation count is exceeded")]
    GenerationCountExceeded,

    #[error(transparent)]
    ImageGenerationFailed(#[from] GatewayError),
}
